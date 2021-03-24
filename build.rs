use anyhow::*;
use shaderc::{
    CompileOptions, Compiler, IncludeType, OptimizationLevel, ResolvedInclude, ShaderKind,
};
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

const OPTLEVEL: OptimizationLevel = OptimizationLevel::Zero;

fn main() -> Result<()> {
    let root_dir = &PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src_dir = &root_dir.join("src/");
    let target_dir = &root_dir.join("target/");

    let mut checksums = ShaderChecksums::load_already_compiled(target_dir);
    let (shaders, include_map) = get_all_shaders(src_dir)?;

    // Shaderc init
    let compile_options = compiler_options(src_dir.clone(), include_map)?;
    let mut compiler = Compiler::new().context("Unable to create shader compiler")?;

    // Preprocess
    let preprocessed_shaders = preprocess_shaders(shaders, &mut compiler, &compile_options)?;

    // Filter unchanged source files after preprocessing
    let shaders_to_compile = preprocessed_shaders
        .into_iter()
        .filter(|shader| checksums.register_new(shader));

    // Compile
    for shader in shaders_to_compile {
        println!("cargo:warning=Compiling shader {}", shader.path_name);

        let compiled = compiler
            .compile_into_spirv(
                &shader.source,
                shader.kind.unwrap(),
                &shader.path_name,
                "main",
                Some(&compile_options),
            )
            .context("While compiling with shaderc")?;

        let extension = match shader.kind.unwrap() {
            ShaderKind::Vertex => "vert",
            ShaderKind::Fragment => "frag",
            ShaderKind::Compute => "comp",
            _ => unreachable!(),
        };

        fs::create_dir_all(target_dir.join(&shader.path_name))
            .with_context(|| format!("While creating shader directory {:?}", &shader.path_name))?;

        fs::write(
            target_dir
                .join(&shader.path_name)
                .with_extension(format!("{}.spv", extension)),
            compiled.as_binary_u8(),
        )
        .with_context(|| {
            format!(
                "While writing the compiled shader to {:?}",
                target_dir.join(&shader.path_name)
            )
        })?;
    }

    // Remember compiled
    checksums.write_file(target_dir);
    Ok(())
}

fn preprocess_shaders(
    shaders: Vec<ShaderSource>,
    compiler: &mut Compiler,
    options: &CompileOptions,
) -> Result<Vec<ShaderSource>> {
    shaders
        .into_iter()
        .map(|shader| {
            Ok(ShaderSource {
                source: compiler
                    .preprocess(&shader.source, &shader.path_name, "main", Some(&options))?
                    .as_text(),
                path_name: shader.path_name,
                kind: shader.kind,
            })
        })
        .collect()
}

fn compiler_options<'a>(
    src_dir: PathBuf,
    include_map: HashMap<String, String>,
) -> Result<CompileOptions<'a>> {
    let mut options = CompileOptions::new().context("While creating shader compile options")?;
    options.set_optimization_level(OPTLEVEL);
    options.set_warnings_as_errors();
    options.set_include_callback(move |to, kind, from, depth| {
        include_callback(&src_dir, &include_map, to, kind, from, depth)
    });
    return Ok(options);

    fn include_callback(
        src_dir: &Path,
        include_map: &HashMap<String, String>,
        to_include: &str,
        include_type: IncludeType,
        from_include: &str,
        depth: usize,
    ) -> Result<ResolvedInclude, String> {
        if depth >= 100 {
            return Err(format!("Bailing due to high include depth (= {})", depth));
        }
        let resolved_name = match include_type {
            IncludeType::Standard => String::from(to_include),

            IncludeType::Relative => {
                let target_path = src_dir
                    .join(from_include)
                    .parent()
                    .ok_or_else(|| {
                        format!("Bad path: {:?} joined onto {:?}", from_include, src_dir)
                    })?
                    .join(to_include);

                match target_path.canonicalize() {
                    Ok(target) => String::from(
                        target
                            .strip_prefix(src_dir)
                            .map_err(|_| {
                                format!(
                                    "Cannot strip {:?} from {:?} (canonicalized from {:?})",
                                    src_dir, target, target_path
                                )
                            })?
                            .to_str()
                            .ok_or_else(|| {
                                format!("Paths must be valid unicode within the repo: {:?}", target)
                            })?,
                    ),

                    Err(ioerror) => {
                        return Err(format!(
                            "Cannot canonicalize path {:?}: {}",
                            target_path, ioerror
                        ))
                    }
                }
            }
        };
        if let Some(content) = include_map.get(&resolved_name) {
            Ok(ResolvedInclude {
                resolved_name,
                content: content.clone(),
            })
        } else {
            Err(format!(
                "Could not find .glsl file at {} relative to src/",
                resolved_name
            ))
        }
    }
}

fn get_all_shaders(src_dir: &Path) -> Result<(Vec<ShaderSource>, HashMap<String, String>)> {
    assert!(src_dir.is_absolute());
    assert!(src_dir.is_dir());

    // [glob::glob] takes a [&str], so we need to change working directory to deal with possible
    // invalid unicode in a parent directory of the repo.
    env::set_current_dir(src_dir)?;
    let paths: Vec<glob::Paths> = ["**/*.vert", "**/*.frag", "**/*.comp", "**/*.glsl"]
        .as_ref()
        .into_iter()
        .map(|glob_str| glob::glob(glob_str).context("Globbing for shaders"))
        .collect::<Result<_>>()?;

    let files: Vec<ShaderSource> = paths
        .into_iter()
        .flatten()
        .map(|glob_result| {
            let absolute_path = src_dir.join(glob_result?);
            ShaderSource::load(absolute_path, src_dir)
        })
        .collect::<Result<_>>()?;

    let (shaders, included): (_, Vec<_>) = files.into_iter().partition(ShaderSource::is_top_level);

    let include_map = included
        .into_iter()
        .map(|source| source.include_pair().unwrap())
        .collect();

    return Ok((shaders, include_map));
}

struct ShaderSource {
    source: String,
    path_name: String, // Relative to src (and after compilation, target) dir
    kind: Option<ShaderKind>,
}

impl ShaderSource {
    pub fn load(path: PathBuf, src_dir: &Path) -> Result<Self> {
        assert!(path.is_absolute());
        assert!(path.is_file());
        assert!(src_dir.is_absolute());
        assert!(src_dir.is_dir());

        let extension = path
            .extension()
            .context("Getting shader file extension")?
            .to_str()
            .context("Getting shader file extension")?;
        let kind = match extension {
            "vert" => Some(ShaderKind::Vertex),
            "frag" => Some(ShaderKind::Fragment),
            "comp" => Some(ShaderKind::Compute),
            "glsl" => None,
            _ => bail!("Unsupported shader: {}", path.display()),
        };

        let source = fs::read_to_string(&path)?;
        let path_name = path
            .strip_prefix(src_dir)
            .context("Shaders must be in src/")?
            .to_str()
            .context("Relative paths within the repo must be valid unicode")?
            .to_string();
        Ok(Self {
            source,
            path_name,
            kind,
        })
    }
    pub fn is_top_level(&self) -> bool {
        self.kind.is_some()
    }
    pub fn include_pair(self) -> Option<(String, String)> {
        if self.kind.is_none() {
            Some((self.path_name, self.source))
        } else {
            None
        }
    }
}

/**
Caches shader source file checksums, to avoid unnecessary recompilation.

Example `target/shader_checksums.txt` content:
```norun
shader.frag bf009481bd9bb7650dcdf903fafc896c
shader.vert 04181d9dc9d21e07dded377e96e6e61b
```
*/
// Tuples (path relative to target dir, md5 digest)
struct ShaderChecksums(HashMap<String, String>);

impl ShaderChecksums {
    fn load_already_compiled(target_dir: &Path) -> Self {
        assert!(target_dir.is_absolute());
        assert!(target_dir.is_dir());

        let entries = match fs::read_to_string(target_dir.join("shader_checksums.txt")) {
            Ok(entries) => entries,
            Err(_) => return Self(HashMap::default()),
        };
        Self(
            entries
                .split('\n')
                .map(|line| {
                    let mut words = line.split(' ');
                    Some((String::from(words.nth(0)?), String::from(words.nth(0)?)))
                })
                .collect::<Option<HashMap<String, String>>>()
                .unwrap_or(HashMap::default()),
        )
    }
    pub fn write_file(self, target_dir: &Path) {
        assert!(target_dir.is_absolute());
        assert!(target_dir.is_dir());

        let entries: Vec<String> = self
            .0
            .into_iter()
            .map(|(path, digest)| format!("{} {}", path, digest))
            .collect();
        fs::write(target_dir.join("shader_checksums.txt"), &entries.join("\n")).unwrap();
    }
    pub fn register_new(&mut self, shader: &ShaderSource) -> bool {
        let digest = format!("{:?}", md5::compute(&shader.source));
        if let Some(old_digest) = self.0.get(&shader.path_name) {
            if *old_digest == digest {
                return false;
            }
        }
        self.0.insert(shader.path_name.clone(), digest);
        return true;
    }
}

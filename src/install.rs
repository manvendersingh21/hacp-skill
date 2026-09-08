use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
const SKILL: &str = include_str!("../skills/hacp/SKILL.md");
const WRAPPER: &str = "---\ndescription: Coordinate with a second coding agent through HACP\n---\nLoad the hacp skill and follow it for this collaboration.\n\n$ARGUMENTS\n";
const CLIS: [&str; 4] = ["claude", "codex", "agy", "opencode"];
#[derive(Clone)]
struct Target {
    path: PathBuf,
    content: &'static str,
}
fn binary(name: &str) -> Option<PathBuf> {
    env::split_paths(&env::var_os("PATH").unwrap_or_default())
        .map(|p| p.join(name))
        .find(|p| p.is_file())
}
fn version(name: &str) -> Result<Option<String>> {
    let Some(path) = binary(name) else {
        return Ok(None);
    };
    let mut child = Command::new(path)
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let started = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if started.elapsed() >= Duration::from_secs(5) {
            child.kill()?;
            child.wait()?;
            return Ok(Some("version command timed out".into()));
        }
        thread::sleep(Duration::from_millis(25));
    }
    let out = child.wait_with_output()?;
    Ok(Some(String::from_utf8_lossy(&out.stdout).trim().into()))
}
fn state(t: &Target) -> Result<&'static str> {
    if t.path.exists() || fs::symlink_metadata(&t.path).is_ok() {
        return Ok(
            if t.path.is_file() && fs::read(&t.path)? == t.content.as_bytes() {
                "identical"
            } else {
                "conflict"
            },
        );
    }
    let mut ancestor = t.path.parent();
    while let Some(p) = ancestor {
        if p.exists() {
            return Ok(
                if p.is_dir() && !fs::metadata(p)?.permissions().readonly() {
                    "missing"
                } else {
                    "conflict"
                },
            );
        }
        ancestor = p.parent();
    }
    Ok("conflict")
}
pub fn run(cli: Option<&str>, home: Option<&Path>, doctor: bool) -> Result<Value> {
    let custom_home = home.is_some();
    let home = home
        .map(Path::to_path_buf)
        .or_else(|| env::var_os("HOME").map(PathBuf::from))
        .context("HOME missing; pass --home")?;
    ensure!(home.is_absolute(), "home must be absolute");
    let config = if custom_home {
        home.join(".config")
    } else {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config"))
    };
    let selected: Vec<_> = match cli {
        Some("all") => CLIS.to_vec(),
        Some(c) => vec![c],
        None => CLIS.into_iter().filter(|c| binary(c).is_some()).collect(),
    };
    ensure!(
        !selected.is_empty(),
        "no supported CLI detected; select --cli explicitly"
    );
    let mut targets: Vec<Target> = vec![];
    let mut reports = vec![];
    for c in &selected {
        let mut paths = vec![];
        let standard = match *c {
            "claude" => home.join(".claude/skills/hacp/SKILL.md"),
            "codex" => home.join(".agents/skills/hacp/SKILL.md"),
            "agy" => home.join(".gemini/config/skills/hacp/SKILL.md"),
            "opencode" => config.join("opencode/skills/hacp/SKILL.md"),
            _ => anyhow::bail!("unsupported CLI {c}"),
        };
        let mut skill_path = standard.clone();
        if *c == "opencode" {
            for p in [
                home.join(".claude/skills/hacp/SKILL.md"),
                home.join(".agents/skills/hacp/SKILL.md"),
                standard.clone(),
            ] {
                if p.exists() || targets.iter().any(|t| t.path == p) {
                    let t = Target {
                        path: p.clone(),
                        content: SKILL,
                    };
                    if state(&t)? == "conflict" {
                        targets.push(t);
                    } else {
                        skill_path = p;
                    }
                }
            }
        }
        paths.push(skill_path.clone());
        targets.push(Target {
            path: skill_path,
            content: SKILL,
        });
        if *c == "opencode" {
            let path = config.join("opencode/commands/hacp.md");
            paths.push(path.clone());
            targets.push(Target {
                path,
                content: WRAPPER,
            });
        }
        reports.push(json!({"cli":c,"version":version(c)?,"paths":paths}));
    }
    targets.sort_by(|a, b| a.path.cmp(&b.path));
    targets.dedup_by(|a, b| a.path == b.path);
    let files: Vec<_> = targets
        .iter()
        .map(|t| Ok(json!({"path":t.path,"status":state(t)?})))
        .collect::<Result<_>>()?;
    let conflicts = files.iter().any(|f| f["status"] == "conflict");
    let prerequisites = json!({"cargo":version("cargo")?,"hacp":binary("hacp"),"shell":Path::new("/bin/sh").is_file()});
    if doctor {
        return Ok(
            json!({"clis":reports,"files":files,"conflicts":conflicts,"prerequisites":prerequisites,"model_calls":0}),
        );
    }
    ensure!(
        !conflicts,
        "installation refused before writing: conflicting destinations {}",
        serde_json::to_string(&files)?
    );
    let mut installed: Vec<PathBuf> = vec![];
    let result = (|| -> Result<()> {
        for t in &targets {
            if state(t)? == "identical" {
                continue;
            }
            let parent = t.path.parent().unwrap();
            fs::create_dir_all(parent)?;
            let tmp = parent.join(format!(".hacp-install-{}", uuid::Uuid::new_v4()));
            let result = (|| -> Result<()> {
                let mut f = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
                f.write_all(t.content.as_bytes())?;
                f.sync_all()?;
                // A hard link publishes the complete file without overwriting a racing writer.
                match fs::hard_link(&tmp, &t.path) {
                    Ok(()) => installed.push(t.path.clone()),
                    Err(e) => {
                        ensure!(
                            state(t)? == "identical",
                            "destination changed during install: {}: {e}",
                            t.path.display()
                        );
                    }
                }
                File::open(parent)?.sync_all()?;
                Ok(())
            })();
            let _ = fs::remove_file(&tmp);
            result?;
        }
        Ok(())
    })();
    if result.is_err() {
        for p in installed {
            let _ = fs::remove_file(p);
        }
        result?;
    }
    Ok(
        json!({"installed":reports,"files":files,"prerequisites":prerequisites,"next":"restart each CLI to discover hacp; use /hacp (Codex: $hacp)"}),
    )
}

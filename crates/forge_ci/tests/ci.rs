use gh_workflow_tailcall::{Input, *};
use indexmap::map::IndexMap;
use serde_json::{json, Value};

#[test]
fn generate() {
    let mut workflow = StandardWorkflow::default()
        .auto_fix(true)
        .add_setup(Step::run("sudo apt-get install -y libsqlite3-dev"))
        .to_ci_workflow()
        .add_env(("FORGE_KEY", "${{secrets.FORGE_KEY}}"));

    // Set up the build matrix for all platforms
    let matrix = json!({
        "include": [
            {
                "os": "ubuntu-latest",
                "target": "x86_64-unknown-linux-gnu",
                "binary_name": "forge-x86_64-unknown-linux-gnu",
                "binary_path": "target/x86_64-unknown-linux-gnu/release/forge"
            },
            {
                "os": "macos-latest",
                "target": "x86_64-apple-darwin",
                "binary_name": "forge-x86_64-apple-darwin",
                "binary_path": "target/x86_64-apple-darwin/release/forge"
            },
            {
                "os": "macos-latest",
                "target": "aarch64-apple-darwin",
                "binary_name": "forge-aarch64-apple-darwin",
                "binary_path": "target/aarch64-apple-darwin/release/forge"
            },
            {
                "os": "windows-latest",
                "target": "x86_64-pc-windows-msvc",
                "binary_name": "forge-x86_64-pc-windows-msvc.exe",
                "binary_path": "target/x86_64-pc-windows-msvc/release/forge.exe"
            }
        ]
    });

    let build_job = workflow.jobs.clone().unwrap().get("build").unwrap().clone();
    let main_cond =
        Expression::new("github.event_name == 'push' && github.ref == 'refs/heads/main'");

    // Add release build job
    workflow = workflow.add_job(
        "build-release",
        Job::new("build-release")
            .add_needs(build_job.clone())
            .cond(main_cond)
            .strategy(Strategy {
                fail_fast: None,
                max_parallel: None,
                matrix: Some(matrix)
            })
            .runs_on("${{ matrix.os }}")
            .add_step(Step::uses("actions", "checkout", "v4"))
            // Install Rust with cross-compilation target
            .add_step(
                Step::uses("dtolnay", "rust-toolchain", "stable")
                    .with(("targets", "${{ matrix.target }}"))
            )
            // Build release binary
            .add_step(
                Step::uses("actions-rs", "cargo", "v1")
                    .with(("command", "build --release --target ${{matrix.target}}"))
            )
            // Create release archive
            .add_step(
                Step::run(r#"
                    cd $(dirname "${{ matrix.binary_path }}")
                    if [ "${{ runner.os }}" = "Windows" ]; then
                        7z a ../../../${{ matrix.binary_name }}.zip $(basename "${{ matrix.binary_path }}")
                    else
                        tar czf ../../../${{ matrix.binary_name }}.tar.gz $(basename "${{ matrix.binary_path }}")
                    fi
                    cd -
                "#)
            )
            // Upload artifact for release
            .add_step(
                {  let indexmap = IndexMap::new();
                    let input = Input::from(indexmap).add("name", "${{ matrix.binary_name }}")
                    .add("path", "${{ inputs.path }}/${{ matrix.binary_name }}.tar.gz")
                    .add("if-no-files-found", "error");
                    Step::uses("actions", "upload-artifact", "v3")
                    .with(input)
                }
            )
    );

    let mut path_map = IndexMap::new();
    path_map.insert(
        "name".to_string(),
        Value::String("${{ matrix.binary_name }}".to_string()),
    );
    path_map.insert(
        "path".to_string(),
        Value::String("${{ inputs.path }}".to_string()),
    );
    // Add release creation job
    let build_release_job = workflow
        .jobs
        .clone()
        .unwrap()
        .get("build-release")
        .unwrap()
        .clone();
    workflow = workflow.add_job(
        "create-release",
        Job::new("create-release")
            .add_needs(build_release_job)
            .runs_on("ubuntu-latest")
            .add_step(Step::uses("actions", "checkout", "v4"))
            // Download all artifacts
            .add_step(Step::uses("actions", "download-artifact", "v3").with(Input::from(path_map)))
            // Create GitHub release
            .add_step(
                Step::uses("softprops", "action-gh-release", "v1")
                    .with(("generate_release_notes", "true"))
                    .with(("files", "${{ inputs.path }}/artifacts/**/*"))
                    .with(("prerelease", "true"))
                    .with(("token", "${{ secrets.GITHUB_TOKEN }}")),
            ),
    );

    workflow.generate().unwrap();
}

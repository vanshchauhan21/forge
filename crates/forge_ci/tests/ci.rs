use gh_workflow_tailcall::*;

#[test]
fn generate() {
    StandardWorkflow::default()
        .auto_fix(true)
        .add_setup(Step::run("sudo apt-get install -y libsqlite3-dev"))
        .to_ci_workflow()
        .add_env(("FORGE_KEY", "${{secrets.FORGE_KEY}}"))
        .generate()
        .unwrap();
}

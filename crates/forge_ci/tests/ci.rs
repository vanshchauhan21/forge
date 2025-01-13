use gh_workflow_tailcall::*;

#[test]
fn generate() {
    StandardWorkflow::default()
        .auto_fix(true)
        .add_setup(Step::run("sudo apt-get install -y libsqlite3-dev"))
        .generate()
        .unwrap();
}

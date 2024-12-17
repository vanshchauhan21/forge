use gh_workflow_tailcall::*;

#[test]
fn generate() {
    Workflow::default().auto_fix(true).generate().unwrap();
}

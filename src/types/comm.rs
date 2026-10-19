#[derive(Debug, Clone)]
pub enum TaskOutput {
    Ok,
}

#[derive(Debug, Clone)]
pub enum InternalMessage {
    Ok,
}

#[derive(Debug, Clone)]
pub enum TaskKind {
    Progress,
}

//process all tasks before drawing frame. Perhaps I could interleave by putting the
//draw call inside this for loop?
//while let Some((task_kind, message)) = self.state.message_stack.pop() {
//    debug!("{task_kind:?}: {message:?}");
//}

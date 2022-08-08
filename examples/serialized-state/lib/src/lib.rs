#[derive(Debug)]
pub struct State {
    pub inner: serde_json::Value,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct InnerState {}

#[no_mangle]
pub fn step(state: State) -> State {
    let inner: InnerState = serde_json::from_value(state.inner).unwrap_or(InnerState {});

    // You can modify the InnerState layout freely and state.inner value here freely!

    State {
        inner: serde_json::to_value(inner).unwrap(),
    }
}

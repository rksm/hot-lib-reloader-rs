#[derive(Debug, Default)]
pub struct State {
    pub version: usize,
    pub inner: Box<serde_json::Value>,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
struct InnerState {}

#[unsafe(no_mangle)]
pub fn step(state: State) -> State {
    let State { version, inner } = state;

    let inner: InnerState = serde_json::from_value(*inner).unwrap_or_default();

    println!("version {version}: {inner:?}");

    // You can modify the InnerState layout freely and state.inner value here freely!

    State {
        version,
        inner: Box::new(serde_json::to_value(inner).unwrap()),
    }
}

use momento_protos::function_types::{CurrentFunctionVersion, WasmId};

/// Description of a Function in Momento.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    name: String,
    description: String,
    function_id: String,
    version: u32,
    latest_version: u32,
}
impl Function {
    /// Name of the function.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Description of the function.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Unique identifier for the function.
    pub fn function_id(&self) -> &str {
        &self.function_id
    }

    /// Currently active version of the function.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Latest available version of the function.
    pub fn latest_version(&self) -> u32 {
        self.latest_version
    }
}

impl From<momento_protos::function_types::Function> for Function {
    fn from(proto: momento_protos::function_types::Function) -> Self {
        let momento_protos::function_types::Function {
            name,
            description,
            function_id,
            current_version,
            latest_version,
        } = proto;
        Self {
            name,
            description,
            function_id,
            version: match current_version {
                Some(CurrentFunctionVersion {
                    version: Some(version),
                }) => match version {
                    momento_protos::function_types::current_function_version::Version::Latest(
                        _latest,
                    ) => latest_version,
                    momento_protos::function_types::current_function_version::Version::Pinned(
                        pinned,
                    ) => pinned.pinned_version,
                },
                _ => u32::MAX,
            },
            latest_version,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentValue {
    /// A literal string environment variable value
    Literal(String),
}

impl From<&str> for EnvironmentValue {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<String> for EnvironmentValue {
    fn from(value: String) -> Self {
        EnvironmentValue::Literal(value)
    }
}

impl From<EnvironmentValue> for momento_protos::function_types::EnvironmentValue {
    fn from(value: EnvironmentValue) -> Self {
        match value {
            EnvironmentValue::Literal(literal) => {
                momento_protos::function_types::EnvironmentValue {
                    value: Some(
                        momento_protos::function_types::environment_value::Value::Literal(literal),
                    ),
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmSource {
    /// The wasm source is right here
    Inline(Vec<u8>),
    /// The wasm source is already in Momento, you just want to reference it by ID
    Reference {
        /// The ID of the wasm source in Momento
        wasm_id: String,
        /// The version of the wasm to refer to
        version: u32,
    },
}

impl From<Vec<u8>> for WasmSource {
    fn from(source: Vec<u8>) -> Self {
        WasmSource::Inline(source)
    }
}

impl From<(String, u32)> for WasmSource {
    fn from((wasm_id, version): (String, u32)) -> Self {
        WasmSource::Reference { wasm_id, version }
    }
}

impl From<(&str, u32)> for WasmSource {
    fn from((wasm_id, version): (&str, u32)) -> Self {
        (wasm_id.to_string(), version).into()
    }
}

impl From<WasmSource> for momento_protos::function::put_function_request::WasmLocation {
    fn from(value: WasmSource) -> Self {
        match value {
            WasmSource::Inline(wasm) => Self::Inline(wasm),
            WasmSource::Reference { wasm_id, version } => Self::WasmId(WasmId {
                id: wasm_id,
                version,
            }),
        }
    }
}

use momento_protos::function_types::WasmId;

/// Description of a Function in Momento.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    name: String,
    description: String,
    function_id: String,
    version: u32,
    latest_version: u32,
    concurrency_limit: u32,
    last_updated_at: String,
    metrics_config: Option<FunctionMetricsConfig>,
}
impl Function {
    /// Name of the function.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Currently active description of the function.
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

    /// The allowed amount of concurrent invocations for this function
    pub fn concurrency_limit(&self) -> u32 {
        self.concurrency_limit
    }

    /// Human-readable timestamp for when the function was last modified.
    /// Will return as a UTC timestamp in ISO 8601 format.
    pub fn last_updated_at(&self) -> &str {
        &self.last_updated_at
    }

    /// Configuration for delivering this function's metrics to your own
    /// CloudWatch account.
    ///
    /// Returns `None` when no configuration is set for this function, in which case it follows
    /// your account-wide default.
    pub fn metrics_config(&self) -> Option<&FunctionMetricsConfig> {
        self.metrics_config.as_ref()
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
            concurrency_limit,
            last_updated_at,
            function_limits: _, // TODO implement getter(s) for function limits
            metrics_config,
        } = proto;
        Self {
            name,
            description,
            function_id,
            metrics_config: metrics_config.and_then(metrics_config_from_proto),
            version: match current_version {
                Some(momento_protos::function_types::CurrentFunctionVersion {
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
            concurrency_limit,
            last_updated_at,
        }
    }
}

/// Per-function configuration for delivering this function's metrics to your own
/// CloudWatch account.
///
/// Set this on a [`PutFunctionRequest`](crate::functions::PutFunctionRequest) or
/// [`PutFunctionConfigRequest`](crate::functions::PutFunctionConfigRequest) to configure metrics
/// for just this function, taking precedence over your account-wide default. When read back from a
/// [`Function`], `None` (an absent configuration) means the function follows your account-wide
/// default.
///
/// Reach out to us at support@momentohq.com to get set up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionMetricsConfig {
    /// Do not deliver this function's metrics to your CloudWatch account.
    Disabled,
    /// Deliver this function's metrics to your CloudWatch account.
    Enabled {
        /// The IAM role Momento should assume to publish metrics into your account.
        customer_iam_role: String,
    },
}

impl FunctionMetricsConfig {
    /// Enable metrics delivery, assuming the given IAM role to publish into your account.
    pub fn enabled(customer_iam_role: impl Into<String>) -> Self {
        FunctionMetricsConfig::Enabled {
            customer_iam_role: customer_iam_role.into(),
        }
    }
}

impl From<FunctionMetricsConfig> for momento_protos::function_types::FunctionMetricsConfig {
    fn from(value: FunctionMetricsConfig) -> Self {
        use momento_protos::function_types::function_metrics_config::{Disabled, Enabled, Kind};
        let kind = match value {
            FunctionMetricsConfig::Disabled => Kind::Disabled(Disabled {}),
            FunctionMetricsConfig::Enabled { customer_iam_role } => {
                Kind::Enabled(Enabled { customer_iam_role })
            }
        };
        Self { kind: Some(kind) }
    }
}

/// Convert a wire-level metrics config into the SDK representation. Returns `None` when the
/// config carries no configuration (an absent inner `kind`), meaning the function follows the
/// account-wide default. A free function rather than a `From` impl because the orphan rule
/// forbids implementing `From<_> for Option<_>` on a foreign source type.
pub(crate) fn metrics_config_from_proto(
    proto: momento_protos::function_types::FunctionMetricsConfig,
) -> Option<FunctionMetricsConfig> {
    use momento_protos::function_types::function_metrics_config::Kind;
    match proto.kind {
        Some(Kind::Disabled(_)) => Some(FunctionMetricsConfig::Disabled),
        Some(Kind::Enabled(enabled)) => Some(FunctionMetricsConfig::Enabled {
            customer_iam_role: enabled.customer_iam_role,
        }),
        None => None,
    }
}

/// A change to a function's per-function metrics-delivery configuration, supplied to
/// [`PutFunctionRequest`](crate::functions::PutFunctionRequest) or
/// [`PutFunctionConfigRequest`](crate::functions::PutFunctionConfigRequest).
///
/// Setting a [`FunctionMetricsConfig`] (via the `From` conversion) configures metrics for just
/// this function, taking precedence over your account-wide default;
/// [`Remove`](FunctionMetricsConfigChange::Remove) clears any existing configuration so the
/// function follows your account-wide default again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionMetricsConfigChange {
    /// Remove any per-function configuration so the function follows your account-wide default.
    Remove,
    /// Set an explicit per-function configuration (enabled or disabled).
    Set(FunctionMetricsConfig),
}

impl From<FunctionMetricsConfig> for FunctionMetricsConfigChange {
    fn from(config: FunctionMetricsConfig) -> Self {
        FunctionMetricsConfigChange::Set(config)
    }
}

impl From<FunctionMetricsConfigChange>
    for momento_protos::function_types::FunctionMetricsConfigChange
{
    fn from(value: FunctionMetricsConfigChange) -> Self {
        use momento_protos::function_types::function_metrics_config_change::{
            Change, RemoveOverride,
        };
        let change = match value {
            FunctionMetricsConfigChange::Remove => Change::RemoveOverride(RemoveOverride {}),
            FunctionMetricsConfigChange::Set(config) => Change::MetricsConfig(config.into()),
        };
        Self {
            change: Some(change),
        }
    }
}

/// A configuration set representing a specific version of a Function in Momento.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionVersion {
    /// The unique identifier for this function version.
    version_id: FunctionVersionId,
    /// The description of the function or this specific version/implementation.
    description: String,
    /// The wasm ID this function uses.
    wasm_version_id: WasmVersionId,
    /// The environment variables available to this function via the WASI environment interface.
    pub environment: std::collections::HashMap<String, EnvironmentValue>,
}
impl FunctionVersion {
    /// The unique identifier for this function version.
    pub fn version_id(&self) -> &FunctionVersionId {
        &self.version_id
    }

    /// The description of the function or this specific version/implementation.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// The wasm ID this function uses.
    pub fn wasm_version_id(&self) -> &WasmVersionId {
        &self.wasm_version_id
    }

    /// The environment variables available to this function via the WASI environment interface.
    pub fn environment(&self) -> &std::collections::HashMap<String, EnvironmentValue> {
        &self.environment
    }
}

impl From<momento_protos::function_types::FunctionVersion> for FunctionVersion {
    fn from(proto: momento_protos::function_types::FunctionVersion) -> Self {
        let momento_protos::function_types::FunctionVersion {
            id,
            description,
            wasm_id,
            environment,
        } = proto;
        let id = id.unwrap_or_default();
        let wasm_id = wasm_id.unwrap_or_default();
        Self {
            version_id: FunctionVersionId {
                id: id.id,
                version: id.version,
            },
            description,
            wasm_version_id: WasmVersionId {
                id: wasm_id.id,
                version: wasm_id.version,
            },
            environment: environment
                .into_iter()
                .map(|(k, v)| (k, EnvironmentValue::from(v)))
                .collect(),
        }
    }
}

/// A unique identifier for a Function in Momento.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionVersionId {
    id: String,
    version: u32,
}
impl FunctionVersionId {
    /// Get the identifier for the function.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the version of the function.
    pub fn version(&self) -> u32 {
        self.version
    }
}

/// A unique identifier for a wasm artifact in Momento.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmVersionId {
    id: String,
    version: u32,
}
impl WasmVersionId {
    /// Get the identifier for the wasm artifact.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the version of the wasm artifact.
    pub fn version(&self) -> u32 {
        self.version
    }
}

/// Metadata about a wasm artifact in Momento.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wasm {
    id: WasmVersionId,
    name: String,
    description: String,
}
impl Wasm {
    /// The unique identifier for this wasm artifact.
    pub fn id(&self) -> &WasmVersionId {
        &self.id
    }

    /// The name of this wasm artifact.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The description of this wasm artifact.
    pub fn description(&self) -> &str {
        &self.description
    }
}

impl From<momento_protos::function_types::Wasm> for Wasm {
    fn from(proto: momento_protos::function_types::Wasm) -> Self {
        let momento_protos::function_types::Wasm {
            id,
            name,
            description,
        } = proto;
        let id = id.unwrap_or_default();
        Self {
            id: WasmVersionId {
                id: id.id,
                version: id.version,
            },
            name,
            description,
        }
    }
}

/// A value for an environment variable in a Momento Function.
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

impl From<momento_protos::function_types::EnvironmentValue> for EnvironmentValue {
    fn from(proto: momento_protos::function_types::EnvironmentValue) -> Self {
        match proto.value {
            Some(value) => match value {
                momento_protos::function_types::environment_value::Value::Literal(literal) => {
                    EnvironmentValue::Literal(literal)
                }
            },
            None => EnvironmentValue::Literal(String::new()),
        }
    }
}

/// The source of the webassembly code for a Momento Function.
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

/// The current version to use upon invocation of the Momento Function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CurrentFunctionVersion {
    /// The latest version
    Latest,
    /// A specific version number
    Pinned(u32),
}

impl From<u32> for CurrentFunctionVersion {
    fn from(pinned_version: u32) -> Self {
        CurrentFunctionVersion::Pinned(pinned_version)
    }
}

impl From<CurrentFunctionVersion> for momento_protos::function_types::CurrentFunctionVersion {
    fn from(value: CurrentFunctionVersion) -> Self {
        match value {
            CurrentFunctionVersion::Latest => {
                momento_protos::function_types::CurrentFunctionVersion {
                    version: Some(
                        momento_protos::function_types::current_function_version::Version::Latest(
                            momento_protos::function_types::current_function_version::Latest {},
                        ),
                    ),
                }
            }
            CurrentFunctionVersion::Pinned(pinned_version) => {
                momento_protos::function_types::CurrentFunctionVersion {
                    version: Some(
                        momento_protos::function_types::current_function_version::Version::Pinned(
                            momento_protos::function_types::current_function_version::Pinned {
                                pinned_version,
                            },
                        ),
                    ),
                }
            }
        }
    }
}

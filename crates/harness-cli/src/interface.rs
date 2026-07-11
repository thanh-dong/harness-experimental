use std::env;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{Args, Parser, Subcommand};
use serde_json::json;
use thiserror::Error;

use crate::application::{
    BacklogAddInput, BacklogCloseInput, BrownfieldImportResult, DecisionAddInput,
    DecisionUpdateInput, HarnessContext,
    HarnessService, InitResult, IntakeInput, InterventionAddInput, InterventionFilter,
    MigrateResult, QueryTable, StoryAddInput, StorySignalAddInput, StorySignalFilter,
    StoryUpdateInput, ToolRegisterInput, TraceInput,
};
use crate::domain::{
    normalize_capability, parse_optional_integer, parse_tool_args, proof_display,
    validate_responsibility, validate_tool_kind, BacklogFilter, BacklogRecord, BoolFlag,
    ContextScoreResult, CsvList, DecisionRecord, FrictionRecord, HarnessStats, ImprovementProposal,
    InputType, IntakeRecord, InterventionRecord, RiskLane, StoryMatrixRecord, StorySignalRecord,
    StoryVerifyAllResult, ToolEntry, TraceQualityTier, TraceRecord, TraceScoreResult,
    RISK_LANE_HELP,
};
use crate::infrastructure::ToolCheckResult;

#[derive(Parser, Debug)]
#[command(name = "harness-cli")]
#[command(about = "durable layer for the project harness", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
    /// Emit a single machine-readable JSON object instead of human text.
    /// Equivalent to setting HARNESS_OUTPUT=json.
    #[arg(long, global = true)]
    json: bool,
}

impl Cli {
    /// Whether `--json` was passed anywhere on the command line.
    pub fn json_output(&self) -> bool {
        self.json
    }
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Create the harness database if it does not already exist.
    Init,
    /// Apply schema migrations.
    Migrate,
    /// Seed or refresh the database from existing markdown state.
    Import(ImportArgs),
    /// Record a feature intake classification.
    Intake(IntakeArgs),
    /// Add or update a story.
    Story(StoryArgs),
    /// Add a decision or run its verification.
    Decision(DecisionArgs),
    /// Add or close a backlog item.
    Backlog(BacklogArgs),
    /// Register or remove external tools.
    Tool(ToolArgs),
    /// Record a human, review, CI, or agent intervention.
    Intervention(InterventionArgs),
    /// Record an agent execution trace.
    Trace(TraceArgs),
    /// Score a trace against the trace quality tiers.
    ScoreTrace(ScoreTraceArgs),
    /// Score trace context reads against CONTEXT_RULES.md.
    ScoreContext { trace_id: String },
    /// Run drift audit and entropy score.
    Audit,
    /// Generate improvement proposals from observed patterns.
    Propose(ProposeArgs),
    /// Verify replay determinism by rebuilding a shadow cache from .harness/events/ (live cache untouched; to repair it, delete harness.db and run any query).
    Rebuild(RebuildArgs),
    /// Migrate an existing database to the event log (proves equality first).
    MigrateToEvents,
    /// Query harness data.
    Query(QueryArgs),
}

#[derive(Args, Debug)]
struct RebuildArgs {
    /// Where to write the rebuilt cache (default: .harness/shadow.db).
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Args, Debug)]
#[command(after_help = RISK_LANE_HELP)]
struct IntakeArgs {
    #[arg(long = "type")]
    input_type: String,
    #[arg(long, allow_hyphen_values = true)]
    summary: String,
    #[arg(long, value_name = "tiny|normal|high-risk")]
    lane: String,
    #[arg(long)]
    flags: Option<String>,
    #[arg(long)]
    docs: Option<String>,
    #[arg(long)]
    story: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    notes: Option<String>,
}

#[derive(Args, Debug)]
struct ImportArgs {
    #[command(subcommand)]
    source: ImportSource,
}

#[derive(Subcommand, Debug)]
enum ImportSource {
    /// Import TEST_MATRIX, decisions, and backlog markdown.
    Brownfield,
}

#[derive(Args, Debug)]
struct StoryArgs {
    #[command(subcommand)]
    action: StoryAction,
}

#[derive(Subcommand, Debug)]
enum StoryAction {
    #[command(after_help = RISK_LANE_HELP)]
    Add(StoryAddArgs),
    #[command(
        after_help = "Proof flags use numeric booleans: --unit 1 --integration 1 --e2e 0 --platform 0. Do not use yes/no."
    )]
    Update(StoryUpdateArgs),
    #[command(
        after_help = "story verify only accepts the story id. Configure proof with story add/update --verify, then record proof flags with story update."
    )]
    Verify {
        /// Story id to verify.
        id: String,
    },
    /// Verify every story, skipping stories without verify_command.
    VerifyAll,
    /// Record an implementation-note signal for propose to mine.
    Signal(SignalArgs),
}

#[derive(Args, Debug)]
struct SignalArgs {
    #[command(subcommand)]
    action: SignalAction,
}

#[derive(Subcommand, Debug)]
enum SignalAction {
    #[command(
        after_help = "Valid --type values: design_decision, deviation, tradeoff, open_question."
    )]
    Add(StorySignalAddArgs),
}

#[derive(Args, Debug)]
struct StorySignalAddArgs {
    #[arg(
        long = "type",
        value_name = "design_decision|deviation|tradeoff|open_question"
    )]
    signal_type: String,
    #[arg(long, allow_hyphen_values = true)]
    summary: String,
    #[arg(long)]
    story: Option<String>,
    #[arg(long)]
    trace: Option<String>,
    #[arg(long)]
    component: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    notes: Option<String>,
}

#[derive(Args, Debug)]
struct StoryAddArgs {
    #[arg(long)]
    id: String,
    #[arg(long)]
    title: String,
    #[arg(long, value_name = "tiny|normal|high-risk")]
    lane: String,
    #[arg(long, allow_hyphen_values = true)]
    contract: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    verify: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    notes: Option<String>,
}

#[derive(Args, Debug)]
struct StoryUpdateArgs {
    #[arg(long)]
    id: String,
    #[arg(long)]
    status: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    evidence: Option<String>,
    #[arg(long, value_name = "0|1")]
    unit: Option<String>,
    #[arg(long, value_name = "0|1")]
    integration: Option<String>,
    #[arg(long, value_name = "0|1")]
    e2e: Option<String>,
    #[arg(long, value_name = "0|1")]
    platform: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    verify: Option<String>,
}

#[derive(Args, Debug)]
struct DecisionArgs {
    #[command(subcommand)]
    action: DecisionAction,
}

#[derive(Subcommand, Debug)]
enum DecisionAction {
    Add(DecisionAddArgs),
    Update(DecisionUpdateArgs),
    Verify { id: String },
}

#[derive(Args, Debug)]
struct DecisionUpdateArgs {
    #[arg(long)]
    id: String,
    #[arg(long)]
    title: Option<String>,
    #[arg(long)]
    status: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    doc: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    verify: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    predicted: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    notes: Option<String>,
}

#[derive(Args, Debug)]
struct DecisionAddArgs {
    #[arg(long)]
    id: String,
    #[arg(long)]
    title: String,
    #[arg(long, default_value = "accepted")]
    status: String,
    #[arg(long, allow_hyphen_values = true)]
    doc: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    verify: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    predicted: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    notes: Option<String>,
}

#[derive(Args, Debug)]
struct BacklogArgs {
    #[command(subcommand)]
    action: BacklogAction,
}

#[derive(Subcommand, Debug)]
enum BacklogAction {
    #[command(after_help = RISK_LANE_HELP)]
    Add(BacklogAddArgs),
    Close(BacklogCloseArgs),
}

#[derive(Args, Debug)]
struct BacklogAddArgs {
    #[arg(long, allow_hyphen_values = true)]
    title: String,
    #[arg(long = "while", allow_hyphen_values = true)]
    discovered_while: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    pain: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    suggestion: Option<String>,
    #[arg(long, value_name = "tiny|normal|high-risk")]
    risk: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    predicted: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    notes: Option<String>,
}

#[derive(Args, Debug)]
struct BacklogCloseArgs {
    #[arg(long)]
    id: String,
    #[arg(long, default_value = "implemented")]
    status: String,
    #[arg(long, allow_hyphen_values = true)]
    outcome: Option<String>,
}

#[derive(Args, Debug)]
struct ToolArgs {
    #[command(subcommand)]
    action: ToolAction,
}

#[derive(Subcommand, Debug)]
enum ToolAction {
    Register(ToolRegisterArgs),
    /// Scan registered tools and persist present/missing/unknown status.
    Check(ToolCheckArgs),
    Remove {
        #[arg(long)]
        name: String,
    },
}

#[derive(Args, Debug)]
struct ToolRegisterArgs {
    #[arg(long)]
    name: String,
    #[arg(long)]
    command: String,
    #[arg(long, allow_hyphen_values = true)]
    description: String,
    #[arg(long)]
    responsibility: String,
    #[arg(long)]
    args: Option<String>,
    #[arg(long)]
    force: bool,
    /// How the tool is reached and probed: cli, binary, mcp, skill, http.
    #[arg(long, default_value = "cli")]
    kind: String,
    /// Workflow purpose a step looks the tool up by (kebab-case).
    #[arg(long)]
    capability: Option<String>,
    /// Declarative path/URL `tool check` resolves to decide presence.
    #[arg(long)]
    scan: Option<String>,
}

#[derive(Args, Debug)]
struct ToolCheckArgs {
    /// Check one tool by name; omit to check every registered tool.
    #[arg(long)]
    name: Option<String>,
}

#[derive(Args, Debug)]
struct InterventionArgs {
    #[command(subcommand)]
    action: InterventionAction,
}

#[derive(Subcommand, Debug)]
enum InterventionAction {
    Add(InterventionAddArgs),
}

#[derive(Args, Debug)]
struct InterventionAddArgs {
    #[arg(long)]
    trace: Option<String>,
    #[arg(long)]
    story: Option<String>,
    #[arg(long = "type")]
    intervention_type: String,
    #[arg(long, allow_hyphen_values = true)]
    description: String,
    #[arg(long)]
    source: String,
    #[arg(long, allow_hyphen_values = true)]
    impact: Option<String>,
}

#[derive(Args, Debug)]
struct TraceArgs {
    #[arg(long, allow_hyphen_values = true)]
    summary: String,
    #[arg(long)]
    intake: Option<String>,
    #[arg(long)]
    story: Option<String>,
    #[arg(long)]
    agent: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    outcome: Option<String>,
    #[arg(long)]
    duration: Option<String>,
    #[arg(long)]
    tokens: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    friction: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    actions: Option<String>,
    #[arg(long = "read", allow_hyphen_values = true)]
    files_read: Option<String>,
    #[arg(long = "changed", allow_hyphen_values = true)]
    files_changed: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    decisions: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    errors: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    notes: Option<String>,
}

#[derive(Args, Debug)]
struct ScoreTraceArgs {
    /// Score a specific trace id. Defaults to the latest trace.
    #[arg(long)]
    id: Option<String>,
}

#[derive(Args, Debug)]
struct ProposeArgs {
    #[arg(long)]
    commit: bool,
}

#[derive(Args, Debug)]
struct QueryArgs {
    #[command(subcommand)]
    view: QueryView,
}

#[derive(Args, Debug)]
struct MatrixQueryArgs {
    /// Render proof flags as CLI input values, 1 and 0, instead of yes and no.
    #[arg(long)]
    numeric: bool,
}

#[derive(Args, Debug)]
struct BacklogQueryArgs {
    /// Show only proposed and accepted backlog items.
    #[arg(long, conflicts_with = "closed")]
    open: bool,
    /// Show only implemented and rejected backlog items.
    #[arg(long)]
    closed: bool,
}

#[derive(Subcommand, Debug)]
enum QueryView {
    /// Test matrix.
    Matrix(MatrixQueryArgs),
    /// Harness improvement proposals.
    Backlog(BacklogQueryArgs),
    /// Decision records.
    Decisions,
    /// Recent intake classifications.
    Intakes,
    /// Recent traces.
    Traces,
    /// Traces with harness friction.
    Friction,
    /// Machine-readable and registered tool manifest.
    Tools(ToolsQueryArgs),
    /// Intervention records.
    Interventions(InterventionsQueryArgs),
    /// Story signals from implementation notes.
    Signals(SignalsQueryArgs),
    /// Summary counts.
    Stats,
    /// Run arbitrary SQL.
    Sql { query: Vec<String> },
}

#[derive(Args, Debug)]
struct ToolsQueryArgs {
    #[arg(long)]
    summary: bool,
    #[arg(long)]
    responsibility: Option<String>,
    /// Filter to tools that provide this capability.
    #[arg(long)]
    capability: Option<String>,
    /// Filter to tools with this scanned status: present, missing, unknown.
    #[arg(long)]
    status: Option<String>,
}

#[derive(Args, Debug)]
struct InterventionsQueryArgs {
    #[arg(long)]
    trace: Option<String>,
    #[arg(long)]
    story: Option<String>,
    #[arg(long = "type")]
    intervention_type: Option<String>,
}

#[derive(Args, Debug)]
struct SignalsQueryArgs {
    #[arg(long)]
    story: Option<String>,
    #[arg(long = "type")]
    signal_type: Option<String>,
}

#[derive(Debug, Error)]
pub enum InterfaceError {
    #[error("{0}")]
    ParseHarnessValue(#[from] crate::domain::ParseHarnessValueError),
    #[error("{0}")]
    ToolValidation(#[from] crate::domain::ToolValidationError),
    #[error("{0}")]
    Infrastructure(#[from] crate::infrastructure::HarnessInfraError),
    #[error("could not determine current directory: {0}")]
    CurrentDir(std::io::Error),
    #[error("query sql requires a SQL statement")]
    EmptySql,
}

/// Whether the CLI renders human text or a single JSON object per invocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputMode {
    Human,
    Json,
}

impl OutputMode {
    /// Resolve the output mode from the `--json` flag and the HARNESS_OUTPUT
    /// environment variable. Either turning it on selects JSON.
    pub fn resolve(json_flag: bool) -> Self {
        let env_json = env::var("HARNESS_OUTPUT")
            .map(|value| value.trim().eq_ignore_ascii_case("json"))
            .unwrap_or(false);
        if json_flag || env_json {
            OutputMode::Json
        } else {
            OutputMode::Human
        }
    }

    fn is_json(self) -> bool {
        matches!(self, OutputMode::Json)
    }
}

impl InterfaceError {
    /// Process exit code: 0 success (never this path), 2 validation / bad input
    /// / not-found (user-correctable), 3 IO or database failure.
    pub fn exit_code(&self) -> i32 {
        match self {
            InterfaceError::ParseHarnessValue(_)
            | InterfaceError::ToolValidation(_)
            | InterfaceError::EmptySql => 2,
            InterfaceError::CurrentDir(_) => 3,
            InterfaceError::Infrastructure(inner) => infra_exit_code(inner),
        }
    }

    /// Stable machine-readable error code string used in the JSON envelope.
    pub fn error_code(&self) -> &'static str {
        match self.exit_code() {
            2 => "validation",
            _ => "io",
        }
    }
}

fn infra_exit_code(error: &crate::infrastructure::HarnessInfraError) -> i32 {
    use crate::infrastructure::HarnessInfraError as E;
    match error {
        // IO / database / internal failures.
        E::Sqlite(_)
        | E::Io(_)
        | E::CorruptEventLog(_, _)
        | E::MigrationVerifyFailed(_) => 3,
        // Everything else is a user-correctable validation / state error.
        _ => 2,
    }
}

/// Render an error as the stable JSON failure envelope.
pub fn error_envelope(error: &InterfaceError) -> String {
    let value = json!({
        "ok": false,
        "error": {
            "code": error.error_code(),
            "message": error.to_string(),
        }
    });
    value.to_string()
}

fn print_json(value: serde_json::Value) {
    println!("{value}");
}

/// Emit `{"ok":true,"command":<command>, ...fields}`.
fn emit_ok(command: &str, mut fields: serde_json::Map<String, serde_json::Value>) {
    fields.insert("ok".to_owned(), serde_json::Value::Bool(true));
    fields.insert(
        "command".to_owned(),
        serde_json::Value::String(command.to_owned()),
    );
    print_json(serde_json::Value::Object(fields));
}

/// Emit `{"ok":true,"command":<command>,"data":<data>}`.
fn emit_data(command: &str, data: serde_json::Value) {
    let mut fields = serde_json::Map::new();
    fields.insert("data".to_owned(), data);
    emit_ok(command, fields);
}

fn ok_fields<const N: usize>(
    pairs: [(&str, serde_json::Value); N],
) -> serde_json::Map<String, serde_json::Value> {
    pairs
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect()
}

pub fn run(cli: Cli, output: OutputMode) -> Result<(), InterfaceError> {
    let service = HarnessService::new(resolve_context()?);

    match cli.command {
        Command::Init => print_init_result(service.init()?),
        Command::Migrate => print_migrate_result(service.migrate()?),
        Command::Import(args) => match args.source {
            ImportSource::Brownfield => {
                print_brownfield_import_result(service.import_brownfield()?)
            }
        },
        Command::Intake(args) => {
            let id = service.record_intake(IntakeInput {
                input_type: InputType::from_str(&args.input_type)?,
                summary: args.summary,
                risk_lane: RiskLane::from_str(&args.lane)?,
                risk_flags: CsvList::from_optional(args.flags),
                affected_docs: CsvList::from_optional(args.docs),
                story_id: args.story,
                notes: args.notes,
            })?;
            if output.is_json() {
                emit_ok("intake", ok_fields([("id", json!(id))]));
            } else {
                println!("Intake {id} recorded.");
            }
        }
        Command::Story(args) => match args.action {
            StoryAction::Add(args) => {
                service.add_story(StoryAddInput {
                    id: args.id.clone(),
                    title: args.title,
                    risk_lane: RiskLane::from_str(&args.lane)?,
                    contract_doc: args.contract,
                    verify_command: args.verify,
                    notes: args.notes,
                })?;
                if output.is_json() {
                    emit_ok("story.add", ok_fields([("id", json!(args.id))]));
                } else {
                    println!("Story {} added.", args.id);
                }
            }
            StoryAction::Update(args) => {
                service.update_story(StoryUpdateInput {
                    id: args.id.clone(),
                    status: args.status,
                    evidence: args.evidence,
                    unit: parse_optional_bool("story update: --unit", args.unit)?,
                    integration: parse_optional_bool(
                        "story update: --integration",
                        args.integration,
                    )?,
                    e2e: parse_optional_bool("story update: --e2e", args.e2e)?,
                    platform: parse_optional_bool("story update: --platform", args.platform)?,
                    verify_command: args.verify,
                })?;
                if output.is_json() {
                    emit_ok("story.update", ok_fields([("id", json!(args.id))]));
                } else {
                    println!("Story {} updated.", args.id);
                }
            }
            StoryAction::Verify { id } => {
                let result = service.verify_story(&id)?;
                println!("Running: {}", result.command);
                print!("{}", result.stdout);
                print!("{}", result.stderr);
                println!("Story {id} verification: {}", result.result);
                if result.result == "fail" {
                    std::process::exit(1);
                }
            }
            StoryAction::VerifyAll => {
                let result = service.verify_all_stories()?;
                print_story_verify_all(&result);
                if result.failed() > 0 {
                    std::process::exit(1);
                }
            }
            StoryAction::Signal(args) => match args.action {
                SignalAction::Add(args) => {
                    let id = service.add_story_signal(StorySignalAddInput {
                        story_id: args.story,
                        trace_id: args.trace,
                        signal_type: args.signal_type,
                        summary: args.summary,
                        component: args.component,
                        notes: args.notes,
                    })?;
                    if output.is_json() {
                        emit_ok("story.signal.add", ok_fields([("id", json!(id))]));
                    } else {
                        println!("Story signal {id} recorded.");
                    }
                }
            },
        },
        Command::Decision(args) => match args.action {
            DecisionAction::Add(args) => {
                service.add_decision(DecisionAddInput {
                    id: args.id.clone(),
                    title: args.title,
                    status: args.status,
                    doc_path: args.doc,
                    verify_command: args.verify,
                    predicted_impact: args.predicted,
                    notes: args.notes,
                })?;
                if output.is_json() {
                    emit_ok("decision.add", ok_fields([("id", json!(args.id))]));
                } else {
                    println!("Decision {} added.", args.id);
                }
            }
            DecisionAction::Update(args) => {
                service.update_decision(DecisionUpdateInput {
                    id: args.id.clone(),
                    title: args.title,
                    status: args.status,
                    doc_path: args.doc,
                    verify_command: args.verify,
                    predicted_impact: args.predicted,
                    notes: args.notes,
                })?;
                println!("Decision {} updated.", args.id);
            }
            DecisionAction::Verify { id } => {
                let result = service.verify_decision(&id)?;
                println!("Running: {}", result.command);
                println!("Decision {id} verification: {}", result.result);
                if result.result == "fail" {
                    std::process::exit(1);
                }
            }
        },
        Command::Backlog(args) => match args.action {
            BacklogAction::Add(args) => {
                let id = service.add_backlog(BacklogAddInput {
                    title: args.title,
                    discovered_while: args.discovered_while,
                    current_pain: args.pain,
                    suggestion: args.suggestion,
                    risk: args
                        .risk
                        .map(|value| RiskLane::from_str(&value))
                        .transpose()?,
                    predicted_impact: args.predicted,
                    notes: args.notes,
                })?;
                if output.is_json() {
                    emit_ok("backlog.add", ok_fields([("id", json!(id))]));
                } else {
                    println!("Backlog {id} added.");
                }
            }
            BacklogAction::Close(args) => {
                let id = args.id;
                let status = args.status;
                service.close_backlog(BacklogCloseInput {
                    id: id.clone(),
                    status: status.clone(),
                    actual_outcome: args.outcome,
                })?;
                println!("Backlog {id} closed as {status}.");
            }
        },
        Command::Tool(args) => match args.action {
            ToolAction::Register(args) => {
                let kind = validate_tool_kind(&args.kind)?;
                let capability = args
                    .capability
                    .as_deref()
                    .map(normalize_capability)
                    .transpose()?;
                service.register_tool(ToolRegisterInput {
                    name: args.name.clone(),
                    command: args.command,
                    description: args.description,
                    responsibility: validate_responsibility(&args.responsibility)?,
                    args: parse_tool_args(args.args)?,
                    force: args.force,
                    kind,
                    capability,
                    scan_target: args.scan,
                })?;
                println!("Tool {} registered.", args.name);
            }
            ToolAction::Check(args) => {
                let results = service.check_tools(args.name)?;
                if output.is_json() {
                    print_tool_check_json(&results);
                } else {
                    print_tool_check_summary(&results);
                }
            }
            ToolAction::Remove { name } => {
                service.remove_tool(&name)?;
                println!("Tool {name} removed.");
            }
        },
        Command::Intervention(args) => match args.action {
            InterventionAction::Add(args) => {
                let id = service.add_intervention(InterventionAddInput {
                    trace_id: args.trace,
                    story_id: args.story,
                    intervention_type: args.intervention_type,
                    description: args.description,
                    source: args.source,
                    impact: args.impact,
                })?;
                println!("Intervention {id} recorded.");
            }
        },
        Command::Trace(args) => {
            let story_id = args.story.clone();
            let id = service.record_trace(TraceInput {
                task_summary: args.summary,
                intake_id: args.intake,
                story_id: args.story,
                agent: args.agent,
                outcome: args.outcome,
                duration_seconds: parse_optional_integer("trace: --duration", args.duration)?,
                token_estimate: parse_optional_integer("trace: --tokens", args.tokens)?,
                friction: args.friction,
                notes: args.notes,
                actions: CsvList::from_optional(args.actions),
                files_read: CsvList::from_optional(args.files_read),
                files_changed: CsvList::from_optional(args.files_changed),
                decisions: CsvList::from_optional(args.decisions),
                errors: CsvList::from_optional(args.errors),
            })?;
            let result = service.score_trace(Some(id.clone()))?;
            if output.is_json() {
                emit_ok(
                    "trace",
                    ok_fields([("id", json!(id)), ("score", trace_score_json(&result))]),
                );
            } else {
                println!("Trace {id} recorded.");
                print_trace_score(&result, false);
                println!(
                    "Reminder: Record any human corrections with: harness-cli intervention add"
                );
                if let Some(story_id) = story_id {
                    print_story_verify_warning(&service, &story_id)?;
                }
            }
        }
        Command::ScoreTrace(args) => {
            let latest = args.id.is_none();
            let result = service.score_trace(args.id)?;
            print_trace_score(&result, latest);
            if !result.meets_requirement {
                std::process::exit(1);
            }
        }
        Command::ScoreContext { trace_id } => {
            print_context_score(&service.score_context(&trace_id)?);
        }
        Command::Audit => print_audit(&service.audit()?),
        Command::Propose(args) => print_proposals(&service.propose(args.commit)?),
        Command::MigrateToEvents => {
            let result = service.migrate_to_events()?;
            if result.already_event_backed {
                println!("Already event-backed; nothing to migrate.");
            } else {
                println!(
                    "Migrated {} rows to .harness/events/migration.jsonl (proof: per-table count + content-hash equality, tool scan columns excluded).",
                    result.events_written
                );
                for (table, count) in &result.table_counts {
                    println!("  {table}: {count}");
                }
                if let Some(backup) = &result.backup_db {
                    println!("old DB backed up at {}", backup.display());
                }
                if result.archived_log_files > 0 {
                    println!(
                        "{} pre-cutover shadow log file(s) archived to .harness/backup/pre-migration-events/",
                        result.archived_log_files
                    );
                }
            }
        }
        Command::Rebuild(args) => {
            let result = service.rebuild(args.output)?;
            println!(
                "Rebuilt {} from {} events.",
                result.db_path.display(),
                result.events_consumed
            );
            for (table, count) in &result.table_counts {
                println!("  {table}: {count}");
            }
            println!("dump hash: {}", result.dump_hash);
        }
        Command::Query(args) => match args.view {
            QueryView::Matrix(args) => {
                let records = service.query_matrix()?;
                if output.is_json() {
                    emit_data(
                        "query.matrix",
                        json!(records.iter().map(matrix_json).collect::<Vec<_>>()),
                    );
                } else {
                    print_matrix(&records, args.numeric);
                }
            }
            QueryView::Backlog(args) => {
                let records = service.query_backlog(backlog_filter(&args))?;
                if output.is_json() {
                    emit_data(
                        "query.backlog",
                        json!(records.iter().map(backlog_json).collect::<Vec<_>>()),
                    );
                } else {
                    print_backlog(&records);
                }
            }
            QueryView::Decisions => {
                let records = service.query_decisions()?;
                if output.is_json() {
                    emit_data(
                        "query.decisions",
                        json!(records.iter().map(decision_json).collect::<Vec<_>>()),
                    );
                } else {
                    print_decisions(&records);
                }
            }
            QueryView::Intakes => {
                let records = service.query_intakes()?;
                if output.is_json() {
                    emit_data(
                        "query.intakes",
                        json!(records.iter().map(intake_json).collect::<Vec<_>>()),
                    );
                } else {
                    print_intakes(&records);
                }
            }
            QueryView::Traces => {
                let records = service.query_traces()?;
                if output.is_json() {
                    emit_data(
                        "query.traces",
                        json!(records.iter().map(trace_json).collect::<Vec<_>>()),
                    );
                } else {
                    print_traces(&records);
                }
            }
            QueryView::Friction => {
                let records = service.query_friction()?;
                if output.is_json() {
                    emit_data(
                        "query.friction",
                        json!(records.iter().map(friction_json).collect::<Vec<_>>()),
                    );
                } else {
                    print_friction(&records);
                }
            }
            QueryView::Tools(args) => {
                let responsibility = args
                    .responsibility
                    .map(|value| validate_responsibility(&value))
                    .transpose()?;
                let capability = args
                    .capability
                    .as_deref()
                    .map(normalize_capability)
                    .transpose()?;
                let mut tools = service.query_tools(responsibility, capability)?;
                if let Some(status) = args.status.as_deref() {
                    let normalized = status.trim().to_lowercase();
                    tools.retain(|tool| tool.status == normalized);
                }
                if output.is_json() {
                    print_tools_json(&tools);
                } else {
                    print_tools_summary(&tools);
                }
            }
            QueryView::Interventions(args) => {
                let records = service.query_interventions(InterventionFilter {
                    trace_id: args.trace,
                    story_id: args.story,
                    intervention_type: args.intervention_type,
                })?;
                if output.is_json() {
                    emit_data(
                        "query.interventions",
                        json!(records.iter().map(intervention_json).collect::<Vec<_>>()),
                    );
                } else {
                    print_interventions(&records);
                }
            }
            QueryView::Signals(args) => {
                let records = service.query_story_signals(StorySignalFilter {
                    story_id: args.story,
                    signal_type: args.signal_type,
                })?;
                if output.is_json() {
                    emit_data(
                        "query.signals",
                        json!(records.iter().map(signal_json).collect::<Vec<_>>()),
                    );
                } else {
                    print_signals(&records);
                }
            }
            QueryView::Stats => {
                let stats = service.query_stats()?;
                if output.is_json() {
                    emit_data("query.stats", stats_json(&stats));
                } else {
                    print_stats(&stats);
                }
            }
            QueryView::Sql { query } => {
                if query.is_empty() {
                    return Err(InterfaceError::EmptySql);
                }
                let table = service.query_sql(&query.join(" "))?;
                if output.is_json() {
                    emit_data("query.sql", query_table_json(&table));
                } else {
                    print_query_table(&table);
                }
            }
        },
    }

    Ok(())
}

fn print_trace_score(result: &TraceScoreResult, latest: bool) {
    if latest {
        println!("Trace #{} (latest):", result.trace_id);
    } else {
        println!("Trace #{}:", result.trace_id);
    }
    println!(
        "  Tier achieved: {} ({}/3)",
        result.achieved.label(),
        result.achieved.score()
    );

    match (&result.risk_lane, result.required) {
        (Some(lane), Some(required)) => {
            println!(
                "  Lane: {} -> required tier: {} ({}/3)",
                lane,
                required.label(),
                required.score()
            );
            if result.meets_requirement {
                println!("  MEETS REQUIREMENT");
            } else {
                println!("  BELOW REQUIREMENT");
            }
        }
        _ => {
            println!("  Lane: unknown (no linked intake)");
        }
    }

    print_missing_fields(
        "minimal",
        TraceQualityTier::Minimal,
        &result.missing_minimal,
    );
    print_missing_fields(
        "standard",
        TraceQualityTier::Standard,
        &result.missing_standard,
    );
    print_missing_fields(
        "detailed",
        TraceQualityTier::Detailed,
        &result.missing_detailed,
    );
}

fn print_story_verify_all(result: &StoryVerifyAllResult) {
    for item in &result.items {
        match item.result.as_str() {
            "skipped" => println!("Story {}: skipped (no verify_command)", item.id),
            status => {
                println!("Story {}: {status}", item.id);
                if !item.stdout.is_empty() {
                    print!("{}", item.stdout);
                }
                if !item.stderr.is_empty() {
                    print!("{}", item.stderr);
                }
            }
        }
    }
    println!(
        "{} stories verified: {} passed, {} failed, {} skipped (no verify_command)",
        result.items.len(),
        result.passed(),
        result.failed(),
        result.skipped()
    );
}

fn print_context_score(result: &ContextScoreResult) {
    println!(
        "Trace #{} | Lane: {} | Phase: {}",
        result.trace_id, result.lane, result.phase
    );
    println!();
    let must_met = result.must.iter().filter(|item| item.met).count();
    println!("Must-read compliance: {must_met}/{}", result.must.len());
    for item in &result.must {
        println!(
            "  {} {} ({})",
            if item.met { "OK" } else { "MISSING" },
            item.label,
            item.target
        );
    }
    let should_met = result.should.iter().filter(|item| item.met).count();
    println!(
        "Should-read compliance: {should_met}/{}",
        result.should.len()
    );
    for item in &result.should {
        println!(
            "  {} {} ({})",
            if item.met { "OK" } else { "MISSING" },
            item.label,
            item.target
        );
    }
    println!("Over-reading: {} item(s)", result.over_read.len());
    for item in &result.over_read {
        println!("  - {item}");
    }
}

fn print_audit(result: &crate::domain::AuditResult) {
    println!("=== Harness Drift Audit ===");
    print_audit_category(
        "Orphaned stories (planned/in-progress, no traces)",
        &result.orphaned_stories,
    );
    print_audit_category("Unverified stories", &result.unverified_stories);
    print_audit_category("Unverified decisions", &result.unverified_decisions);
    print_audit_category(
        "Open backlog without outcomes",
        &result.backlog_without_outcomes,
    );
    print_audit_category("Stale stories", &result.stale_stories);
    print_audit_category("Broken tools", &result.broken_tools);
    print_audit_category(
        "Concurrent LWW updates (causal audit)",
        &result.concurrent_lww_updates,
    );
    println!(
        "Entropy score: {}/100 (lower is better)",
        result.entropy_score()
    );
}

fn print_audit_category(label: &str, findings: &[crate::domain::AuditFinding]) {
    println!();
    println!("{label}: {}", findings.len());
    for finding in findings {
        println!("  - {}: {}", finding.id, finding.title);
    }
}

fn print_proposals(proposals: &[ImprovementProposal]) {
    println!("=== Improvement Proposals ===");
    if proposals.is_empty() {
        println!("No proposals generated.");
        return;
    }
    for (index, proposal) in proposals.iter().enumerate() {
        println!();
        println!(
            "Proposal {} ({} confidence):",
            index + 1,
            proposal.confidence
        );
        println!("  Title: {}", proposal.title);
        println!("  Component: {}", proposal.component);
        println!("  Evidence: {}", proposal.evidence);
        println!("  Predicted impact: {}", proposal.predicted_impact);
        println!("  Risk: {}", proposal.risk);
        println!("  Suggested action: {}", proposal.suggested_action);
        println!("  Validation: {}", proposal.validation_plan);
        if let Some(id) = &proposal.committed_backlog_id {
            println!("  Created backlog item #{id}");
        }
    }
    println!();
    println!(
        "{} proposals generated. Use --commit to create backlog items.",
        proposals.len()
    );
}

fn print_story_verify_warning(
    service: &HarnessService,
    story_id: &str,
) -> Result<(), InterfaceError> {
    let status = service.story_verify_status(story_id)?;
    let has_command = status
        .verify_command
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    if has_command && status.last_verified_result.as_deref() != Some("pass") {
        println!();
        println!(
            "Warning: Story {} has verify_command but verification has not passed.",
            status.id
        );
        println!("Run: harness-cli story verify {}", status.id);
    }
    Ok(())
}

fn print_missing_fields(label: &str, tier: TraceQualityTier, fields: &[String]) {
    if fields.is_empty() {
        return;
    }
    println!();
    println!("  Missing for {label}:");
    for field in fields {
        println!("    - {field}");
    }
    if tier == TraceQualityTier::Detailed {
        println!();
    }
}

fn backlog_filter(args: &BacklogQueryArgs) -> BacklogFilter {
    if args.open {
        BacklogFilter::Open
    } else if args.closed {
        BacklogFilter::Closed
    } else {
        BacklogFilter::All
    }
}

fn print_brownfield_import_result(result: BrownfieldImportResult) {
    println!("Brownfield import complete.");
    println!("Stories imported or updated: {}", result.stories);
    println!("Decisions imported or updated: {}", result.decisions);
    println!("Backlog items discovered: {}", result.backlog_items);
}

fn parse_optional_bool(
    label: &str,
    value: Option<String>,
) -> Result<Option<BoolFlag>, InterfaceError> {
    value
        .map(|inner| BoolFlag::parse(label, &inner))
        .transpose()
        .map_err(InterfaceError::from)
}

fn print_init_result(result: InitResult) {
    match result {
        InitResult::Created { db_path } => {
            println!("Creating harness database at {}", db_path.display());
            println!("Schema applied.");
        }
        InitResult::Existing { db_path, version } => {
            println!("Database already exists at {}", db_path.display());
            println!("Current schema version: {version}");
        }
        InitResult::MigratedExisting { db_path } => {
            println!("Database already exists at {}", db_path.display());
            println!("No schema version found. Applying schema.");
            println!("Schema applied.");
        }
    }
}

fn print_migrate_result(result: MigrateResult) {
    println!("Current schema version: {}", result.current_version);
    if result.applied.is_empty() {
        println!("Already up to date.");
    } else {
        for version in &result.applied {
            println!("Applying migration {version}...");
        }
        println!("Applied {} migration(s).", result.applied.len());
    }
}

fn resolve_context() -> Result<HarnessContext, InterfaceError> {
    let repo_root = match env::var_os("HARNESS_REPO_ROOT") {
        Some(path) => PathBuf::from(path),
        None => env::current_dir().map_err(InterfaceError::CurrentDir)?,
    };
    let db_path = env::var_os("HARNESS_DB")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join("harness.db"));

    let schema_dir = repo_root.join("scripts/schema");

    Ok(HarnessContext {
        repo_root,
        db_path,
        schema_dir,
    })
}

fn print_matrix(records: &[StoryMatrixRecord], numeric: bool) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.id.clone(),
                record.title.clone(),
                record.status.clone(),
                proof_display(record.unit, numeric),
                proof_display(record.integration, numeric),
                proof_display(record.e2e, numeric),
                proof_display(record.platform, numeric),
                record.evidence.clone().unwrap_or_default(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(
        &[
            "id", "title", "status", "unit", "integ", "e2e", "plat", "evidence",
        ],
        &rows,
    );
}

fn print_backlog(records: &[BacklogRecord]) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.id.to_string(),
                record.title.clone(),
                record.status.clone(),
                record.risk.clone().unwrap_or_default(),
                record.predicted_impact.clone().unwrap_or_default(),
                record.actual_outcome.clone().unwrap_or_default(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(
        &[
            "id",
            "title",
            "status",
            "risk",
            "predicted_impact",
            "actual_outcome",
        ],
        &rows,
    );
}

fn print_decisions(records: &[DecisionRecord]) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.id.clone(),
                record.title.clone(),
                record.status.clone(),
                record.last_verified_at.clone().unwrap_or_default(),
                record.last_verified_result.clone().unwrap_or_default(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(
        &[
            "id",
            "title",
            "status",
            "last_verified_at",
            "last_verified_result",
        ],
        &rows,
    );
}

fn print_intakes(records: &[IntakeRecord]) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.id.to_string(),
                record.created_at.clone(),
                record.input_type.clone(),
                record.risk_lane.clone(),
                record.summary.clone(),
            ]
        })
        .collect::<Vec<_>>();

    print_table(
        &["id", "created_at", "input_type", "risk_lane", "summary"],
        &rows,
    );
}

fn print_traces(records: &[TraceRecord]) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.id.to_string(),
                record.created_at.clone(),
                record.outcome.clone().unwrap_or_default(),
                record.task_summary.clone(),
                record.harness_friction.clone().unwrap_or_default(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(
        &[
            "id",
            "created_at",
            "outcome",
            "task_summary",
            "harness_friction",
        ],
        &rows,
    );
}

fn print_friction(records: &[FrictionRecord]) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.id.to_string(),
                record.created_at.clone(),
                record.risk_lane.clone().unwrap_or_else(|| "-".to_owned()),
                record.input_type.clone().unwrap_or_else(|| "-".to_owned()),
                record.task_summary.clone(),
                record.harness_friction.clone(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(
        &[
            "id",
            "created_at",
            "risk_lane",
            "input_type",
            "task_summary",
            "harness_friction",
        ],
        &rows,
    );
}

fn print_tools_summary(records: &[ToolEntry]) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.name.clone(),
                record.kind.clone(),
                record.capability.clone().unwrap_or_else(|| "-".to_owned()),
                record.responsibility.clone(),
                record.status.clone(),
                record.source.clone(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(
        &[
            "name",
            "kind",
            "capability",
            "responsibility",
            "status",
            "source",
        ],
        &rows,
    );
}

fn print_tools_json(records: &[ToolEntry]) {
    println!("[");
    for (index, record) in records.iter().enumerate() {
        let comma = if index + 1 == records.len() { "" } else { "," };
        println!("  {{");
        println!("    \"provider\": \"{}\",", json_escape(&record.provider));
        println!("    \"name\": \"{}\",", json_escape(&record.name));
        println!("    \"command\": \"{}\",", json_escape(&record.command));
        println!(
            "    \"description\": \"{}\",",
            json_escape(&record.description)
        );
        println!("    \"args\": [");
        for (arg_index, arg) in record.args.iter().enumerate() {
            let arg_comma = if arg_index + 1 == record.args.len() {
                ""
            } else {
                ","
            };
            println!(
                "      {{\"name\":\"{}\",\"type\":\"{}\",\"required\":{},\"help\":\"{}\"}}{}",
                json_escape(&arg.name),
                json_escape(&arg.arg_type),
                arg.required,
                json_escape(arg.help.as_deref().unwrap_or("")),
                arg_comma
            );
        }
        println!("    ],");
        println!(
            "    \"responsibility\": \"{}\",",
            json_escape(&record.responsibility)
        );
        println!("    \"source\": \"{}\",", json_escape(&record.source));
        println!("    \"since\": \"{}\",", json_escape(&record.since));
        println!("    \"kind\": \"{}\",", json_escape(&record.kind));
        println!(
            "    \"capability\": {},",
            json_optional(record.capability.as_deref())
        );
        println!(
            "    \"scan_target\": {},",
            json_optional(record.scan_target.as_deref())
        );
        println!("    \"status\": \"{}\",", json_escape(&record.status));
        println!(
            "    \"checked_at\": {}",
            json_optional(record.checked_at.as_deref())
        );
        println!("  }}{comma}");
    }
    println!("]");
}

fn print_tool_check_summary(records: &[ToolCheckResult]) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.name.clone(),
                record.kind.clone(),
                record.capability.clone().unwrap_or_else(|| "-".to_owned()),
                record.status.clone(),
                record.detail.clone(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(&["name", "kind", "capability", "status", "detail"], &rows);
}

fn print_tool_check_json(records: &[ToolCheckResult]) {
    println!("[");
    for (index, record) in records.iter().enumerate() {
        let comma = if index + 1 == records.len() { "" } else { "," };
        println!("  {{");
        println!("    \"name\": \"{}\",", json_escape(&record.name));
        println!("    \"kind\": \"{}\",", json_escape(&record.kind));
        println!(
            "    \"capability\": {},",
            json_optional(record.capability.as_deref())
        );
        println!("    \"status\": \"{}\",", json_escape(&record.status));
        println!("    \"detail\": \"{}\"", json_escape(&record.detail));
        println!("  }}{comma}");
    }
    println!("]");
}

fn json_optional(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("\"{}\"", json_escape(value)),
        None => "null".to_owned(),
    }
}

fn print_interventions(records: &[InterventionRecord]) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.id.to_string(),
                record.created_at.clone(),
                record
                    .trace_id
                    .clone()
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                record.story_id.clone().unwrap_or_default(),
                record.intervention_type.clone(),
                record.source.clone(),
                record.description.clone(),
                record.impact.clone().unwrap_or_default(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(
        &[
            "id",
            "created_at",
            "trace",
            "story",
            "type",
            "source",
            "description",
            "impact",
        ],
        &rows,
    );
}

fn print_signals(records: &[StorySignalRecord]) {
    let rows = records
        .iter()
        .map(|record| {
            vec![
                record.id.to_string(),
                record.created_at.clone(),
                record.story_id.clone().unwrap_or_default(),
                record.signal_type.clone(),
                record.summary.clone(),
                record.component.clone().unwrap_or_default(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(
        &["id", "created_at", "story", "type", "summary", "component"],
        &rows,
    );
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn matrix_json(record: &StoryMatrixRecord) -> serde_json::Value {
    json!({
        "id": record.id,
        "title": record.title,
        "status": record.status,
        "unit": record.unit != 0,
        "integration": record.integration != 0,
        "e2e": record.e2e != 0,
        "platform": record.platform != 0,
        "evidence": record.evidence,
    })
}

fn backlog_json(record: &BacklogRecord) -> serde_json::Value {
    json!({
        "id": record.id,
        "title": record.title,
        "status": record.status,
        "risk": record.risk,
        "predicted_impact": record.predicted_impact,
        "actual_outcome": record.actual_outcome,
    })
}

fn decision_json(record: &DecisionRecord) -> serde_json::Value {
    json!({
        "id": record.id,
        "title": record.title,
        "status": record.status,
        "last_verified_at": record.last_verified_at,
        "last_verified_result": record.last_verified_result,
    })
}

fn intake_json(record: &IntakeRecord) -> serde_json::Value {
    json!({
        "id": record.id,
        "created_at": record.created_at,
        "input_type": record.input_type,
        "risk_lane": record.risk_lane,
        "summary": record.summary,
    })
}

fn trace_json(record: &TraceRecord) -> serde_json::Value {
    json!({
        "id": record.id,
        "created_at": record.created_at,
        "outcome": record.outcome,
        "task_summary": record.task_summary,
        "harness_friction": record.harness_friction,
    })
}

fn friction_json(record: &FrictionRecord) -> serde_json::Value {
    json!({
        "id": record.id,
        "created_at": record.created_at,
        "risk_lane": record.risk_lane,
        "input_type": record.input_type,
        "task_summary": record.task_summary,
        "harness_friction": record.harness_friction,
    })
}

fn intervention_json(record: &InterventionRecord) -> serde_json::Value {
    json!({
        "id": record.id,
        "created_at": record.created_at,
        "trace_id": record.trace_id,
        "story_id": record.story_id,
        "intervention_type": record.intervention_type,
        "description": record.description,
        "source": record.source,
        "impact": record.impact,
    })
}

fn signal_json(record: &StorySignalRecord) -> serde_json::Value {
    json!({
        "id": record.id,
        "created_at": record.created_at,
        "story_id": record.story_id,
        "trace_id": record.trace_id,
        "signal_type": record.signal_type,
        "summary": record.summary,
        "component": record.component,
        "notes": record.notes,
    })
}

fn stats_json(stats: &HarnessStats) -> serde_json::Value {
    json!({
        "intakes": stats.intakes,
        "stories": stats.stories,
        "decisions": stats.decisions,
        "backlog_items": stats.backlog_items,
        "traces": stats.traces,
    })
}

fn query_table_json(table: &QueryTable) -> serde_json::Value {
    let rows = table
        .rows
        .iter()
        .map(|row| {
            let mut object = serde_json::Map::new();
            for (index, header) in table.headers.iter().enumerate() {
                let cell = row.get(index).cloned().unwrap_or_default();
                object.insert(header.clone(), json!(cell));
            }
            serde_json::Value::Object(object)
        })
        .collect::<Vec<_>>();
    json!(rows)
}

fn trace_score_json(result: &TraceScoreResult) -> serde_json::Value {
    json!({
        "trace_id": result.trace_id,
        "achieved_tier": result.achieved.label(),
        "achieved_score": result.achieved.score(),
        "risk_lane": result.risk_lane,
        "required_tier": result.required.map(|tier| tier.label()),
        "required_score": result.required.map(|tier| tier.score()),
        "meets_requirement": result.meets_requirement,
        "missing_minimal": result.missing_minimal,
        "missing_standard": result.missing_standard,
        "missing_detailed": result.missing_detailed,
    })
}

fn print_stats(stats: &HarnessStats) {
    println!("=== Harness Stats ===");
    print_table(
        &["intakes", "stories", "decisions", "backlog_items", "traces"],
        &[vec![
            stats.intakes.to_string(),
            stats.stories.to_string(),
            stats.decisions.to_string(),
            stats.backlog_items.to_string(),
            stats.traces.to_string(),
        ]],
    );
}

fn print_query_table(table: &QueryTable) {
    let headers = table.headers.iter().map(String::as_str).collect::<Vec<_>>();
    print_table(&headers, &table.rows);
}

fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    let widths = headers
        .iter()
        .enumerate()
        .map(|(index, header)| {
            rows.iter()
                .filter_map(|row| row.get(index))
                .map(String::len)
                .chain(std::iter::once(header.len()))
                .max()
                .unwrap_or(header.len())
        })
        .collect::<Vec<_>>();

    print_row(
        &headers
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>(),
        &widths,
    );
    print_row(
        &widths
            .iter()
            .map(|width| "-".repeat(*width))
            .collect::<Vec<_>>(),
        &widths,
    );
    for row in rows {
        print_row(row, &widths);
    }
}

fn print_row(values: &[String], widths: &[usize]) {
    for (index, width) in widths.iter().enumerate() {
        if index > 0 {
            print!("  ");
        }
        let value = values.get(index).map(String::as_str).unwrap_or("");
        print!("{value:<width$}");
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    fn json_service() -> (tempfile::TempDir, HarnessService) {
        let temp = tempfile::tempdir().unwrap();
        let schema_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .join("scripts/schema");
        let service = HarnessService::new(HarnessContext {
            repo_root: temp.path().to_path_buf(),
            db_path: temp.path().join("harness.db"),
            schema_dir,
        });
        service.init().unwrap();
        (temp, service)
    }

    #[test]
    fn output_mode_selects_json_from_flag_or_env() {
        assert_eq!(OutputMode::resolve(true), OutputMode::Json);
        assert_eq!(OutputMode::resolve(false), OutputMode::Human);
        // The env var is process-global; assert only the flag path here to
        // avoid cross-test interference.
    }

    #[test]
    fn validation_errors_exit_two_io_errors_exit_three() {
        let parse: InterfaceError = RiskLane::from_str("bogus").unwrap_err().into();
        assert_eq!(parse.exit_code(), 2);
        assert_eq!(parse.error_code(), "validation");

        let empty_sql = InterfaceError::EmptySql;
        assert_eq!(empty_sql.exit_code(), 2);

        let not_found: InterfaceError =
            crate::infrastructure::HarnessInfraError::StoryNotFound("US-1".to_owned()).into();
        assert_eq!(not_found.exit_code(), 2);

        let io: InterfaceError =
            crate::infrastructure::HarnessInfraError::Io(std::io::Error::other("disk gone")).into();
        assert_eq!(io.exit_code(), 3);
        assert_eq!(io.error_code(), "io");
    }

    #[test]
    fn error_envelope_is_stable_failure_shape() {
        let error: InterfaceError = InterfaceError::EmptySql;
        let value: serde_json::Value = serde_json::from_str(&error_envelope(&error)).unwrap();
        assert_eq!(value["ok"], serde_json::json!(false));
        assert_eq!(value["error"]["code"], serde_json::json!("validation"));
        assert!(value["error"]["message"].is_string());
    }

    #[test]
    fn matrix_json_shape_is_stable() {
        let record = StoryMatrixRecord {
            id: "US-1".to_owned(),
            title: "title".to_owned(),
            status: "planned".to_owned(),
            unit: 1,
            integration: 0,
            e2e: 0,
            platform: 0,
            evidence: None,
        };
        assert_eq!(
            matrix_json(&record),
            json!({
                "id": "US-1",
                "title": "title",
                "status": "planned",
                "unit": true,
                "integration": false,
                "e2e": false,
                "platform": false,
                "evidence": null,
            })
        );
    }

    #[test]
    fn backlog_round_trips_through_json() {
        let (_temp, service) = json_service();
        let id = service
            .add_backlog(BacklogAddInput {
                title: "round-trip via --json".to_owned(),
                discovered_while: None,
                current_pain: None,
                suggestion: None,
                risk: None,
                predicted_impact: None,
                notes: None,
            })
            .unwrap();

        let records = service.query_backlog(BacklogFilter::All).unwrap();
        let data = json!(records.iter().map(backlog_json).collect::<Vec<_>>());
        assert_eq!(data[0]["id"], json!(id));
        assert_eq!(data[0]["title"], json!("round-trip via --json"));
        assert_eq!(data[0]["status"], json!("proposed"));
    }

    #[test]
    fn intake_round_trips_through_json() {
        let (_temp, service) = json_service();
        let id = service
            .record_intake(IntakeInput {
                input_type: InputType::from_str("change_request").unwrap(),
                summary: "add json mode".to_owned(),
                risk_lane: RiskLane::from_str("normal").unwrap(),
                risk_flags: CsvList::from_optional(None),
                affected_docs: CsvList::from_optional(None),
                story_id: None,
                notes: None,
            })
            .unwrap();

        let records = service.query_intakes().unwrap();
        let data = json!(records.iter().map(intake_json).collect::<Vec<_>>());
        assert_eq!(data[0]["id"], json!(id));
        assert_eq!(data[0]["input_type"], json!("change_request"));
        assert_eq!(data[0]["risk_lane"], json!("normal"));
        assert_eq!(data[0]["summary"], json!("add json mode"));
    }

    #[test]
    fn story_help_documents_proof_command_shape() {
        let mut command = Cli::command();
        let story = command.find_subcommand_mut("story").unwrap();

        let update_help = story
            .find_subcommand_mut("update")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(update_help.contains("--unit <0|1>"));
        assert!(update_help.contains("--integration <0|1>"));
        assert!(update_help.contains("Proof flags use numeric booleans"));

        let verify_help = story
            .find_subcommand_mut("verify")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(verify_help.contains("story verify only accepts the story id"));
        assert!(verify_help.contains("Configure proof with story add/update --verify"));
    }

    #[test]
    fn command_help_documents_lane_values_and_version() {
        let mut command = Cli::command();
        assert!(command.render_long_help().to_string().contains("--version"));

        let intake_help = command
            .find_subcommand_mut("intake")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(intake_help.contains("--lane <tiny|normal|high-risk>"));
        assert!(intake_help.contains("Use tiny instead of low"));

        let story_add_help = command
            .find_subcommand_mut("story")
            .unwrap()
            .find_subcommand_mut("add")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(story_add_help.contains("--lane <tiny|normal|high-risk>"));

        let backlog_add_help = command
            .find_subcommand_mut("backlog")
            .unwrap()
            .find_subcommand_mut("add")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(backlog_add_help.contains("--risk <tiny|normal|high-risk>"));
        assert!(backlog_add_help.contains("Accepted lanes"));

        let matrix_help = command
            .find_subcommand_mut("query")
            .unwrap()
            .find_subcommand_mut("matrix")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(matrix_help.contains("--numeric"));
    }
}

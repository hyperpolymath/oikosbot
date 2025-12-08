// Core types for Eco-Bot analysis

type codeLocation = {
  file: string,
  line: int,
  column: option<int>,
}

type ecoMetrics = {
  carbonScore: float,
  energyScore: float,
  resourceScore: float,
  score: float,
}

type paretoStatus = {
  isOptimal: bool,
  distance: float,
  improvements: option<array<string>>,
}

type econMetrics = {
  paretoDistance: float,
  allocationScore: float,
  debtScore: float,
  score: float,
  paretoStatus: option<paretoStatus>,
}

type qualityMetrics = {
  complexityScore: float,
  couplingScore: float,
  coverageScore: float,
  score: float,
}

type healthIndex = {
  eco: float,
  econ: float,
  quality: float,
  total: float,
  grade: string,
}

type severity =
  | Blocking
  | High
  | Medium
  | Low
  | Info

type policyViolation = {
  entityId: string,
  policy: string,
  severity: severity,
  message: string,
  location: option<codeLocation>,
  suggestions: array<string>,
}

type priority =
  | PriorityHigh
  | PriorityMedium
  | PriorityLow

type recommendation = {
  entityId: string,
  action: string,
  reason: string,
  priority: priority,
  confidence: float,
  expectedImprovement: Js.Dict.t<float>,
}

type analysisResult = {
  eco: ecoMetrics,
  econ: econMetrics,
  quality: qualityMetrics,
  health: healthIndex,
  violations: array<policyViolation>,
  recommendations: array<recommendation>,
  timestamp: string,
  commitSha: option<string>,
}

// Bot modes
type botMode =
  | Consultant // Answers questions, provides alternatives
  | Advisor // Proactive suggestions on PRs
  | Regulator // Enforces policy compliance

// Webhook event types
type platform =
  | GitHub
  | GitLab

type webhookEvent = {
  platform: platform,
  eventType: string,
  action: option<string>,
  repository: {
    owner: string,
    name: string,
    url: string,
  },
  payload: Js.Json.t,
}

// Configuration
type config = {
  port: int,
  mode: botMode,
  analysisEndpoint: string,
  githubWebhookSecret: option<string>,
  gitlabWebhookSecret: option<string>,
}

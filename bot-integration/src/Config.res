// Configuration loading from environment

open Types

let getEnv = (key: string, ~default: option<string>=?): option<string> => {
  switch Deno.Env.get(key) {
  | Some(v) => Some(v)
  | None => default
  }
}

let getEnvRequired = (key: string): result<string, string> => {
  switch Deno.Env.get(key) {
  | Some(v) => Ok(v)
  | None => Error(`Missing required environment variable: ${key}`)
  }
}

let getEnvInt = (key: string, ~default: int): int => {
  switch Deno.Env.get(key) {
  | Some(v) =>
    switch Belt.Int.fromString(v) {
    | Some(i) => i
    | None => default
    }
  | None => default
  }
}

let parseMode = (s: string): botMode => {
  switch Js.String.toLowerCase(s) {
  | "consultant" => Consultant
  | "regulator" => Regulator
  | _ => Advisor
  }
}

let load = (): result<config, string> => {
  let mode = getEnv("BOT_MODE", ~default=Some("advisor"))->Belt.Option.getExn->parseMode

  let analysisEndpoint = switch getEnv("ANALYSIS_ENDPOINT") {
  | Some(e) => e
  | None => "http://localhost:8080/analyze"
  }

  Ok({
    port: getEnvInt("PORT", ~default=3000),
    mode,
    analysisEndpoint,
    githubWebhookSecret: getEnv("GITHUB_WEBHOOK_SECRET"),
    gitlabWebhookSecret: getEnv("GITLAB_WEBHOOK_SECRET"),
  })
}

let modeToString = (mode: botMode): string => {
  switch mode {
  | Consultant => "consultant"
  | Advisor => "advisor"
  | Regulator => "regulator"
  }
}

;;; STATE.scm - Eco-Bot Project State Checkpoint
;;; GNU Guile Scheme format for AI conversation continuity
;;; Upload at session start, download at session end

(define state
  '(
    ;;=========================================================================
    ;; METADATA
    ;;=========================================================================
    (metadata
     (format-version . "1.0.0")
     (project-name . "git-eco-bot")
     (created . "2025-12-08")
     (last-updated . "2025-12-08")
     (session-id . "01MHeccuL6ntJTbdvhw5uxEz")
     (branch . "claude/create-state-scm-01MHeccuL6ntJTbdvhw5uxEz"))

    ;;=========================================================================
    ;; CURRENT POSITION
    ;;=========================================================================
    (current-position
     (phase . "foundation-complete")
     (overall-completion . 45)
     (summary . "Architecture fully documented. Core type systems defined across
                 Haskell/ReScript/Python. Bot integration functional. Multiple
                 analyzers and engines at skeleton/partial state. No tests.
                 Components not yet wired together end-to-end.")

     (components
      ((name . "architecture-docs")
       (status . "complete")
       (completion . 100)
       (notes . "README.md and ARCHITECTURE.md fully documented"))

      ((name . "haskell-analyzer")
       (status . "in-progress")
       (completion . 40)
       (implemented . ("Types.Metrics" "Eco.Carbon" "Eco.Pareto"))
       (missing . ("Energy.hs" "Resource.hs" "Quality.Complexity"
                   "Quality.Coupling" "Quality.Debt" "Quality.Coverage"
                   "Main.hs entry point")))

      ((name . "rescript-bot")
       (status . "in-progress")
       (completion . 85)
       (implemented . ("Main.res" "Webhook.res" "Report.res" "Types.res"
                       "Analysis.res" "Config.res" "Deno bindings"))
       (missing . ("Live analysis integration" "Real webhook verification")))

      ((name . "ocaml-docs-analyzer")
       (status . "skeleton")
       (completion . 5)
       (implemented . ("dune-project"))
       (missing . ("NLP semantic extraction" "Markdown parser"
                   "Claim extraction" "All implementation")))

      ((name . "policy-engine")
       (status . "in-progress")
       (completion . 60)
       (implemented . ("policy_engine.py structure" "eco_rules.dl"
                       "eco_problog.pl" "Engine class skeletons"))
       (missing . ("Live Souffle execution" "Live DeepProbLog execution"
                   "Knowledge graph integration")))

      ((name . "databases")
       (status . "schema-only")
       (completion . 50)
       (implemented . ("ArangoDB schema" "Virtuoso RDF ontology"))
       (missing . ("Connection layer" "Sync logic" "Migration scripts")))

      ((name . "containers")
       (status . "ready")
       (completion . 80)
       (implemented . ("Containerfile" "Containerfile.policy"
                       "compose.yaml" "nerdctl-build.sh"))
       (missing . ("Multi-stage build testing" "Production hardening")))

      ((name . "rust-orchestrator")
       (status . "not-started")
       (completion . 0)
       (notes . "Mentioned in architecture, no files exist"))

      ((name . "testing")
       (status . "not-started")
       (completion . 0)
       (notes . "No test suites implemented"))))

    ;;=========================================================================
    ;; ROUTE TO MVP v1
    ;;=========================================================================
    (mvp-v1-route
     (target . "Minimal viable eco-bot that can analyze a PR and post comments")
     (estimated-completion . 65)  ; percent of work remaining

     (phases
      ((phase . 1)
       (name . "Complete Haskell Analyzer Core")
       (priority . "critical")
       (tasks
        ("Implement src/Eco/Energy.hs - energy pattern analysis"
         "Implement src/Eco/Resource.hs - resource utilization"
         "Implement src/Quality/Complexity.hs - cyclomatic complexity"
         "Create Main.hs with CLI entry point"
         "Add JSON output for bot consumption")))

      ((phase . 2)
       (name . "Wire Bot to Real Analysis")
       (priority . "critical")
       (tasks
        ("Replace mock Analysis.res with subprocess calls to Haskell"
         "Implement proper HMAC signature verification in Webhook.res"
         "Add error handling and retry logic"
         "Test with real GitHub webhook events")))

      ((phase . 3)
       (name . "Basic Policy Enforcement")
       (priority . "high")
       (tasks
        ("Wire Souffle execution in policy_engine.py"
         "Implement threshold-based blocking rules"
         "Add pass/fail/warn status to bot reports")))

      ((phase . 4)
       (name . "End-to-End Integration")
       (priority . "high")
       (tasks
        ("Create integration script connecting all components"
         "Build and test container images"
         "Write deployment instructions"
         "Manual end-to-end test with real repository")))

      ((phase . 5)
       (name . "Basic Testing")
       (priority . "medium")
       (tasks
        ("Add Haskell HSpec tests for analyzer"
         "Add ReScript tests for webhook parsing"
         "Add Python pytest for policy engine"
         "Create example test repository"))))

     (mvp-deliverables
      ("GitHub App that responds to PR events"
       "Carbon intensity score on PRs"
       "Pass/fail based on configurable thresholds"
       "Human-readable PR comments with recommendations")))

    ;;=========================================================================
    ;; ISSUES & BLOCKERS
    ;;=========================================================================
    (issues
     (blockers
      ((id . "BLOCK-001")
       (severity . "high")
       (title . "No executable entry point")
       (description . "Haskell analyzer has no Main.hs - cannot be run as CLI
                       or subprocess by the bot integration layer")
       (impact . "Bot cannot perform real analysis")
       (resolution . "Implement Main.hs with CLI argument parsing"))

      ((id . "BLOCK-002")
       (severity . "high")
       (title . "Mock analysis returns static data")
       (description . "Analysis.res returns hardcoded mock data instead of
                       calling real analyzers")
       (impact . "All PR comments show fake metrics")
       (resolution . "Implement subprocess spawning to Haskell analyzer")))

     (technical-debt
      ((id . "DEBT-001")
       (title . "OCaml analyzer is empty shell")
       (description . "Only dune-project exists, no implementation")
       (deferrable . #t)
       (notes . "Can ship MVP without documentation analysis"))

      ((id . "DEBT-002")
       (title . "No database connections implemented")
       (description . "Schema exists but no code to connect/query")
       (deferrable . #t)
       (notes . "MVP can work without persistence, add later"))

      ((id . "DEBT-003")
       (title . "Rust orchestrator not started")
       (description . "Architecture mentions Rust for orchestration, none exists")
       (deferrable . #t)
       (notes . "Can use simpler orchestration for MVP"))

      ((id . "DEBT-004")
       (title . "DeepProbLog not wired")
       (description . "Probabilistic learning rules exist but not executed")
       (deferrable . #t)
       (notes . "Deterministic Datalog sufficient for MVP")))

     (uncertainties
      ((id . "UNC-001")
       (title . "Haskell-to-ReScript IPC mechanism")
       (description . "How should ReScript bot spawn and communicate with
                       Haskell analyzer? Options: subprocess with JSON stdout,
                       HTTP API, shared file, message queue")
       (needs-decision . #t))

      ((id . "UNC-002")
       (title . "Deployment target")
       (description . "Where will this run? Self-hosted container, cloud
                       function, GitHub Actions runner?")
       (needs-decision . #t))

      ((id . "UNC-003")
       (title . "Carbon estimation data sources")
       (description . "SCI calculation requires grid carbon intensity data.
                       What API/database to use? Electricity Maps? Static?")
       (needs-decision . #t))))

    ;;=========================================================================
    ;; QUESTIONS FOR USER
    ;;=========================================================================
    (questions
     ((id . "Q-001")
      (category . "architecture")
      (question . "What IPC mechanism do you prefer between ReScript/Deno bot
                   and Haskell analyzer?")
      (options . ("subprocess with JSON stdout (simplest)"
                  "HTTP API (more flexible, more complexity)"
                  "shared temporary files (simple, less clean)"
                  "message queue like NATS (enterprise-grade, most complex)"))
      (recommendation . "subprocess with JSON stdout for MVP"))

     ((id . "Q-002")
      (category . "deployment")
      (question . "What is the target deployment environment?")
      (options . ("Self-hosted container (Docker/nerdctl)"
                  "Cloud functions (AWS Lambda, GCP Functions)"
                  "Kubernetes cluster"
                  "GitHub Actions runner"
                  "Hybrid approach"))
      (impacts . "Affects container design, startup time requirements,
                  persistence strategy"))

     ((id . "Q-003")
      (category . "scope")
      (question . "For MVP v1, should we skip OCaml docs analyzer entirely
                   and focus on code-only analysis?")
      (recommendation . "Yes - documentation analysis is valuable but not
                         essential for proving core value proposition"))

     ((id . "Q-004")
      (category . "scope")
      (question . "Should MVP include GitLab support or focus on GitHub only?")
      (recommendation . "GitHub only for MVP - GitLab is implemented in
                         ReScript but adds testing burden"))

     ((id . "Q-005")
      (category . "data")
      (question . "For carbon intensity calculations, what grid data source
                   should we use?")
      (options . ("Electricity Maps API (paid, accurate, real-time)"
                  "Static regional averages (free, less accurate)"
                  "User-provided configuration value"
                  "WattTime API (alternative provider)"))
      (impacts . "Affects accuracy of SCI calculations and operating costs"))

     ((id . "Q-006")
      (category . "testing")
      (question . "Do you have a test repository we can use for integration
                   testing, or should we create one?")
      (notes . "Need real GitHub App registration for end-to-end testing"))

     ((id . "Q-007")
      (category . "features")
      (question . "Which bot mode should MVP default to?")
      (options . ("consultant - answer questions only"
                  "advisor - proactive suggestions on PRs"
                  "regulator - enforce policies, can block merges"))
      (recommendation . "advisor - demonstrates value without being intrusive")))

    ;;=========================================================================
    ;; LONG-TERM ROADMAP
    ;;=========================================================================
    (roadmap
     (vision . "Make ecological and economic thinking first-class citizens in
                software development, creating a feedback loop where code
                quality, environmental impact, and economic efficiency are
                continuously measured and improved.")

     (milestones
      ((version . "v0.1-mvp")
       (name . "Proof of Concept")
       (status . "in-progress")
       (goals . ("GitHub PR analysis with carbon scoring"
                 "Basic pass/fail thresholds"
                 "Human-readable PR comments"
                 "Containerized deployment"))
       (metrics . ("Analyze 1 real PR end-to-end"
                   "< 30s analysis time"
                   "Accurate carbon estimation within 20%")))

      ((version . "v0.2")
       (name . "Policy Foundation")
       (status . "planned")
       (goals . ("Full Datalog policy engine"
                 "Configurable organizational policies"
                 "Block/warn/approve workflow integration"
                 "Basic persistence in ArangoDB"))
       (depends-on . ("v0.1-mvp")))

      ((version . "v0.3")
       (name . "Learning Loop")
       (status . "planned")
       (goals . ("DeepProbLog probabilistic learning"
                 "Learn from merge outcomes"
                 "Improve predictions over time"
                 "Knowledge graph population"))
       (depends-on . ("v0.2")))

      ((version . "v0.4")
       (name . "Documentation Intelligence")
       (status . "planned")
       (goals . ("OCaml NLP documentation analyzer"
                 "Claim extraction from READMEs"
                 "Verify claims against code behavior"
                 "Documentation quality scoring"))
       (depends-on . ("v0.3")))

      ((version . "v0.5")
       (name . "Formal Verification")
       (status . "planned")
       (goals . ("Echidna integration for smart contracts"
                 "Property-based testing automation"
                 "Verified eco/econ claims"
                 "Audit-ready reports"))
       (depends-on . ("v0.4")))

      ((version . "v1.0")
       (name . "Production Ready")
       (status . "planned")
       (goals . ("Multi-tenant SaaS deployment"
                 "Enterprise GitHub App"
                 "Self-hosted option with Kubernetes"
                 "Comprehensive documentation"
                 "99.9% uptime SLA"))
       (depends-on . ("v0.5")))

      ((version . "v2.0")
       (name . "Ecosystem Integration")
       (status . "vision")
       (goals . ("IDE plugins (VS Code, JetBrains)"
                 "CI/CD pipeline integration"
                 "Carbon budgeting and forecasting"
                 "Industry benchmarking"
                 "Regulatory compliance reporting (EU CSRD)"))))

     (research-tracks
      ((name . "Carbon Modeling Accuracy")
       (description . "Improve accuracy of code-to-carbon estimation")
       (approaches . ("Runtime instrumentation"
                      "Hardware performance counters"
                      "ML-based prediction from AST patterns"
                      "Benchmark suite calibration")))

      ((name . "Multi-Language Support")
       (description . "Extend analysis beyond initial languages")
       (priority-languages . ("Rust" "Go" "Java" "C++" "Python"))
       (approach . "Language-agnostic AST intermediate representation"))

      ((name . "Economic Modeling")
       (description . "Better technical debt and cost estimation")
       (approaches . ("Historical data mining"
                      "Team velocity correlation"
                      "Maintenance cost prediction")))))

    ;;=========================================================================
    ;; RECENT HISTORY
    ;;=========================================================================
    (history
     (commits
      ((hash . "f9787ce")
       (date . "2025-12-08")
       (message . "refactor: Replace TypeScript/npm with ReScript/Deno"))
      ((hash . "99ed6d5")
       (date . "2025-12-08")
       (message . "refactor: Replace TypeScript/npm with ReScript/Deno"))
      ((hash . "d36cbf1")
       (date . "2025-12-08")
       (message . "feat: Initial eco-bot platform architecture")))

     (session-notes
      ("2025-12-08: Created STATE.scm checkpoint system"
       "Architecture fully documented in README.md and ARCHITECTURE.md"
       "Migrated from TypeScript/npm to ReScript/Deno for cleaner runtime"
       "Added Guix and Nix package management support")))

    ;;=========================================================================
    ;; NEXT ACTIONS (PRIORITIZED)
    ;;=========================================================================
    (next-actions
     ((priority . 1)
      (action . "Create Main.hs entry point for Haskell analyzer")
      (why . "Unblocks all integration work")
      (effort . "small"))

     ((priority . 2)
      (action . "Implement Energy.hs and remaining Haskell modules")
      (why . "Complete core analysis capability")
      (effort . "medium"))

     ((priority . 3)
      (action . "Wire Analysis.res to call Haskell subprocess")
      (why . "Replace mocks with real analysis")
      (effort . "small"))

     ((priority . 4)
      (action . "Add basic HSpec tests for Haskell analyzer")
      (why . "Ensure correctness before integration")
      (effort . "medium"))

     ((priority . 5)
      (action . "End-to-end container build and test")
      (why . "Validate deployment model")
      (effort . "medium"))

     ((priority . 6)
      (action . "Create test GitHub repository and register App")
      (why . "Enable real-world testing")
      (effort . "small")))

    )) ; end state

;;; Usage:
;;; - Load with: (load "STATE.scm")
;;; - Access with: (assoc 'current-position state)
;;; - Query with: (assoc-ref (assoc 'mvp-v1-route state) 'phases)

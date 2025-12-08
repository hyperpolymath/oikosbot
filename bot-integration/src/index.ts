/**
 * Eco-Bot Integration Layer
 *
 * Provides GitHub/GitLab integration for the Eco-Bot platform.
 * Acts as consultant, advisor, regulator, and policy developer
 * for software repositories.
 */

import express from 'express';
import { Octokit } from '@octokit/rest';
import { Webhooks } from '@octokit/webhooks';
import { Gitlab } from '@gitbeaker/rest';
import pino from 'pino';
import { z } from 'zod';
import type { AnalysisResult, PolicyViolation, Recommendation } from './types';

const logger = pino({ level: process.env.LOG_LEVEL || 'info' });

// =============================================================================
// Configuration
// =============================================================================

const ConfigSchema = z.object({
  port: z.number().default(3000),
  github: z.object({
    appId: z.string(),
    privateKey: z.string(),
    webhookSecret: z.string(),
  }).optional(),
  gitlab: z.object({
    token: z.string(),
    webhookSecret: z.string(),
  }).optional(),
  analysisEndpoint: z.string().default('http://localhost:8080/analyze'),
  mode: z.enum(['consultant', 'advisor', 'regulator']).default('advisor'),
});

type Config = z.infer<typeof ConfigSchema>;

// =============================================================================
// Bot Modes
// =============================================================================

enum BotMode {
  /** Answers questions, provides alternatives, explains trade-offs */
  Consultant = 'consultant',
  /** Proactive suggestions on PRs, best practice recommendations */
  Advisor = 'advisor',
  /** Enforces policy compliance, can block PRs */
  Regulator = 'regulator',
}

// =============================================================================
// Analysis Service Client
// =============================================================================

class AnalysisClient {
  constructor(private endpoint: string) {}

  async analyzeRepository(repoUrl: string, ref: string): Promise<AnalysisResult> {
    const response = await fetch(`${this.endpoint}/repository`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ url: repoUrl, ref }),
    });

    if (!response.ok) {
      throw new Error(`Analysis failed: ${response.statusText}`);
    }

    return response.json();
  }

  async analyzeDiff(repoUrl: string, base: string, head: string): Promise<AnalysisResult> {
    const response = await fetch(`${this.endpoint}/diff`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ url: repoUrl, base, head }),
    });

    if (!response.ok) {
      throw new Error(`Diff analysis failed: ${response.statusText}`);
    }

    return response.json();
  }
}

// =============================================================================
// Report Generator
// =============================================================================

class ReportGenerator {
  /**
   * Generate a PR comment with analysis results
   */
  generatePRComment(analysis: AnalysisResult, mode: BotMode): string {
    const { eco, econ, quality, health, violations, recommendations } = analysis;

    let comment = `## 🌱 Eco-Bot Analysis\n\n`;

    // Health Index Badge
    const grade = this.getGrade(health.total);
    const gradeEmoji = this.getGradeEmoji(grade);
    comment += `### Overall Health: ${gradeEmoji} ${grade} (${health.total.toFixed(1)}/100)\n\n`;

    // Score breakdown
    comment += `| Metric | Score | Status |\n`;
    comment += `|--------|-------|--------|\n`;
    comment += `| 🌍 Ecological | ${eco.score.toFixed(1)} | ${this.getStatusEmoji(eco.score)} |\n`;
    comment += `| 📊 Economic | ${econ.score.toFixed(1)} | ${this.getStatusEmoji(econ.score)} |\n`;
    comment += `| ⚙️ Quality | ${quality.score.toFixed(1)} | ${this.getStatusEmoji(quality.score)} |\n\n`;

    // Violations (if any)
    if (violations.length > 0) {
      comment += `### ⚠️ Policy Violations\n\n`;
      for (const v of violations) {
        const icon = v.severity === 'blocking' ? '🚫' : '⚠️';
        comment += `${icon} **${v.policy}**: ${v.message}\n`;
      }
      comment += '\n';
    }

    // Recommendations (limited in advisor mode)
    if (recommendations.length > 0 && mode !== BotMode.Regulator) {
      const maxRecs = mode === BotMode.Consultant ? 10 : 5;
      const topRecs = recommendations.slice(0, maxRecs);

      comment += `### 💡 Recommendations\n\n`;
      for (const r of topRecs) {
        const confidence = (r.confidence * 100).toFixed(0);
        comment += `- **${r.action}** (${confidence}% confidence): ${r.reason}\n`;
        if (r.expectedImprovement && Object.keys(r.expectedImprovement).length > 0) {
          const improvements = Object.entries(r.expectedImprovement)
            .map(([k, v]) => `${k}: +${v}`)
            .join(', ');
          comment += `  - Expected improvement: ${improvements}\n`;
        }
      }
      comment += '\n';
    }

    // Pareto status
    if (econ.paretoStatus) {
      comment += `### 📈 Pareto Analysis\n\n`;
      if (econ.paretoStatus.isOptimal) {
        comment += `✅ This code is on the Pareto frontier - no dominated trade-offs detected.\n\n`;
      } else {
        comment += `📍 Distance from Pareto frontier: ${econ.paretoStatus.distance.toFixed(2)}\n`;
        if (econ.paretoStatus.improvements && econ.paretoStatus.improvements.length > 0) {
          comment += `\nPotential Pareto improvements:\n`;
          for (const imp of econ.paretoStatus.improvements) {
            comment += `- ${imp}\n`;
          }
        }
        comment += '\n';
      }
    }

    // Footer
    comment += `---\n`;
    comment += `*Analyzed by [Eco-Bot](https://github.com/hyperpolymath/eco-bot) | `;
    comment += `Mode: ${mode} | `;
    comment += `[Learn more about eco-friendly coding](https://greensoftware.foundation/)*\n`;

    return comment;
  }

  /**
   * Generate SARIF output for integration with GitHub Code Scanning
   */
  generateSARIF(analysis: AnalysisResult): object {
    return {
      $schema: 'https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json',
      version: '2.1.0',
      runs: [{
        tool: {
          driver: {
            name: 'eco-bot',
            version: '0.1.0',
            informationUri: 'https://github.com/hyperpolymath/eco-bot',
            rules: this.generateSARIFRules(),
          },
        },
        results: analysis.violations.map(v => ({
          ruleId: `eco/${v.policy.replace(/_/g, '-')}`,
          level: v.severity === 'blocking' ? 'error' : 'warning',
          message: { text: v.message },
          locations: v.location ? [{
            physicalLocation: {
              artifactLocation: { uri: v.location.file },
              region: { startLine: v.location.line },
            },
          }] : [],
          properties: {
            ecoScore: analysis.eco.score,
            econScore: analysis.econ.score,
            suggestions: v.suggestions,
          },
        })),
      }],
    };
  }

  private generateSARIFRules(): object[] {
    return [
      {
        id: 'eco/eco-minimum',
        name: 'EcoMinimum',
        shortDescription: { text: 'Eco minimum threshold not met' },
        fullDescription: { text: 'Component does not meet minimum ecological standards' },
        defaultConfiguration: { level: 'error' },
      },
      {
        id: 'eco/eco-standard',
        name: 'EcoStandard',
        shortDescription: { text: 'Eco standard threshold not met' },
        fullDescription: { text: 'Component does not meet recommended ecological standards' },
        defaultConfiguration: { level: 'warning' },
      },
      {
        id: 'eco/high-carbon',
        name: 'HighCarbon',
        shortDescription: { text: 'High carbon intensity detected' },
        fullDescription: { text: 'Code has high estimated carbon intensity' },
        defaultConfiguration: { level: 'warning' },
      },
    ];
  }

  private getGrade(score: number): string {
    if (score >= 90) return 'A';
    if (score >= 80) return 'B';
    if (score >= 70) return 'C';
    if (score >= 60) return 'D';
    return 'F';
  }

  private getGradeEmoji(grade: string): string {
    const emojis: Record<string, string> = {
      'A': '🏆',
      'B': '✨',
      'C': '👍',
      'D': '⚠️',
      'F': '🚨',
    };
    return emojis[grade] || '📊';
  }

  private getStatusEmoji(score: number): string {
    if (score >= 70) return '✅';
    if (score >= 50) return '⚠️';
    return '❌';
  }
}

// =============================================================================
// GitHub Integration
// =============================================================================

class GitHubBot {
  private octokit: Octokit;
  private webhooks: Webhooks;
  private analysisClient: AnalysisClient;
  private reportGenerator: ReportGenerator;
  private mode: BotMode;

  constructor(config: Config) {
    if (!config.github) {
      throw new Error('GitHub configuration required');
    }

    this.octokit = new Octokit({ auth: config.github.privateKey });
    this.webhooks = new Webhooks({ secret: config.github.webhookSecret });
    this.analysisClient = new AnalysisClient(config.analysisEndpoint);
    this.reportGenerator = new ReportGenerator();
    this.mode = config.mode as BotMode;

    this.setupWebhooks();
  }

  private setupWebhooks(): void {
    // Handle PR opened/synchronized
    this.webhooks.on(['pull_request.opened', 'pull_request.synchronize'], async ({ payload }) => {
      const { repository, pull_request: pr } = payload;
      const owner = repository.owner.login;
      const repo = repository.name;

      logger.info({ owner, repo, pr: pr.number }, 'Analyzing PR');

      try {
        // Analyze the diff
        const analysis = await this.analysisClient.analyzeDiff(
          repository.clone_url,
          pr.base.sha,
          pr.head.sha
        );

        // Generate and post comment
        const comment = this.reportGenerator.generatePRComment(analysis, this.mode);
        await this.octokit.issues.createComment({
          owner,
          repo,
          issue_number: pr.number,
          body: comment,
        });

        // In regulator mode, set check status
        if (this.mode === BotMode.Regulator) {
          const hasBlockingViolations = analysis.violations.some(v => v.severity === 'blocking');
          await this.octokit.checks.create({
            owner,
            repo,
            name: 'Eco-Bot Policy Check',
            head_sha: pr.head.sha,
            status: 'completed',
            conclusion: hasBlockingViolations ? 'failure' : 'success',
            output: {
              title: hasBlockingViolations ? 'Policy Violations Found' : 'All Policies Passed',
              summary: `Health Index: ${analysis.health.total.toFixed(1)}/100`,
            },
          });
        }

        // Upload SARIF for code scanning integration
        const sarif = this.reportGenerator.generateSARIF(analysis);
        // Note: Would upload to /repos/{owner}/{repo}/code-scanning/sarifs

        logger.info({ owner, repo, pr: pr.number }, 'Analysis complete');
      } catch (error) {
        logger.error({ error, owner, repo, pr: pr.number }, 'Analysis failed');
      }
    });

    // Handle issue comments (for consultant mode questions)
    this.webhooks.on('issue_comment.created', async ({ payload }) => {
      const { repository, issue, comment } = payload;
      const body = comment.body.toLowerCase();

      // Check if it's a question for eco-bot
      if (body.includes('@eco-bot') || body.includes('/eco')) {
        const owner = repository.owner.login;
        const repo = repository.name;

        logger.info({ owner, repo, issue: issue.number }, 'Handling eco-bot mention');

        // Parse the question and generate response
        // This would integrate with the policy engine for answers
      }
    });
  }

  getMiddleware(): express.RequestHandler {
    return this.webhooks.middleware;
  }
}

// =============================================================================
// GitLab Integration
// =============================================================================

class GitLabBot {
  private gitlab: InstanceType<typeof Gitlab>;
  private analysisClient: AnalysisClient;
  private reportGenerator: ReportGenerator;
  private mode: BotMode;

  constructor(config: Config) {
    if (!config.gitlab) {
      throw new Error('GitLab configuration required');
    }

    this.gitlab = new Gitlab({ token: config.gitlab.token });
    this.analysisClient = new AnalysisClient(config.analysisEndpoint);
    this.reportGenerator = new ReportGenerator();
    this.mode = config.mode as BotMode;
  }

  async handleMergeRequest(projectId: number, mrIid: number): Promise<void> {
    const mr = await this.gitlab.MergeRequests.show(projectId, mrIid);

    const analysis = await this.analysisClient.analyzeDiff(
      mr.web_url.replace(/\/-\/merge_requests\/\d+$/, ''),
      mr.diff_refs?.base_sha || '',
      mr.diff_refs?.head_sha || ''
    );

    const comment = this.reportGenerator.generatePRComment(analysis, this.mode);

    await this.gitlab.MergeRequestNotes.create(projectId, mrIid, comment);
  }

  getWebhookHandler(): express.RequestHandler {
    return async (req, res) => {
      const event = req.headers['x-gitlab-event'];

      if (event === 'Merge Request Hook') {
        const { project, object_attributes: mr } = req.body;
        if (['open', 'update'].includes(mr.action)) {
          await this.handleMergeRequest(project.id, mr.iid);
        }
      }

      res.status(200).send('OK');
    };
  }
}

// =============================================================================
// Main Application
// =============================================================================

async function main(): Promise<void> {
  const config = ConfigSchema.parse({
    port: parseInt(process.env.PORT || '3000', 10),
    github: process.env.GITHUB_APP_ID ? {
      appId: process.env.GITHUB_APP_ID,
      privateKey: process.env.GITHUB_PRIVATE_KEY,
      webhookSecret: process.env.GITHUB_WEBHOOK_SECRET,
    } : undefined,
    gitlab: process.env.GITLAB_TOKEN ? {
      token: process.env.GITLAB_TOKEN,
      webhookSecret: process.env.GITLAB_WEBHOOK_SECRET,
    } : undefined,
    analysisEndpoint: process.env.ANALYSIS_ENDPOINT,
    mode: process.env.BOT_MODE as 'consultant' | 'advisor' | 'regulator',
  });

  const app = express();
  app.use(express.json());

  // Health check
  app.get('/health', (req, res) => {
    res.json({ status: 'healthy', mode: config.mode });
  });

  // GitHub webhooks
  if (config.github) {
    const githubBot = new GitHubBot(config);
    app.use('/webhooks/github', githubBot.getMiddleware());
    logger.info('GitHub integration enabled');
  }

  // GitLab webhooks
  if (config.gitlab) {
    const gitlabBot = new GitLabBot(config);
    app.post('/webhooks/gitlab', gitlabBot.getWebhookHandler());
    logger.info('GitLab integration enabled');
  }

  // Start server
  app.listen(config.port, () => {
    logger.info({ port: config.port, mode: config.mode }, 'Eco-Bot started');
  });
}

main().catch(error => {
  logger.error({ error }, 'Failed to start Eco-Bot');
  process.exit(1);
});

/**
 * Type definitions for Eco-Bot integration
 */

export interface CodeLocation {
  file: string;
  line: number;
  column?: number;
}

export interface EcoMetrics {
  carbonScore: number;
  energyScore: number;
  resourceScore: number;
  score: number;
}

export interface ParetoStatus {
  isOptimal: boolean;
  distance: number;
  improvements?: string[];
}

export interface EconMetrics {
  paretoDistance: number;
  allocationScore: number;
  debtScore: number;
  score: number;
  paretoStatus?: ParetoStatus;
}

export interface QualityMetrics {
  complexityScore: number;
  couplingScore: number;
  coverageScore: number;
  score: number;
}

export interface HealthIndex {
  eco: number;
  econ: number;
  quality: number;
  total: number;
  grade: string;
}

export interface PolicyViolation {
  entityId: string;
  policy: string;
  severity: 'blocking' | 'high' | 'medium' | 'low' | 'info';
  message: string;
  location?: CodeLocation;
  suggestions: string[];
}

export interface Recommendation {
  entityId: string;
  action: string;
  reason: string;
  priority: 'high' | 'medium' | 'low';
  confidence: number;
  expectedImprovement: Record<string, number>;
}

export interface AnalysisResult {
  eco: EcoMetrics;
  econ: EconMetrics;
  quality: QualityMetrics;
  health: HealthIndex;
  violations: PolicyViolation[];
  recommendations: Recommendation[];
  timestamp: string;
  commitSha?: string;
}

export interface AnalysisRequest {
  url: string;
  ref?: string;
  base?: string;
  head?: string;
}

export interface WebhookEvent {
  type: 'pull_request' | 'push' | 'issue_comment' | 'merge_request';
  action: string;
  repository: {
    owner: string;
    name: string;
    url: string;
  };
  payload: unknown;
}

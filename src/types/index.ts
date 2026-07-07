export interface SkillSource {
  from: string;
  path: string;
  name: string;
  description: string;
  is_symlink: boolean;
  symlink_target?: string;
  content_hash: string;
}

export interface Skill {
  id: string;
  name: string;
  description: string;
  path: string;
  linked_agents: string[];
  sources: SkillSource[];
  license?: string;
  compatibility?: string;
  metadata?: Record<string, string>;
  has_scripts: boolean;
  has_references: boolean;
  has_assets: boolean;
  modified_at: string;
  has_conflict: boolean;
  has_dangling: boolean;
  is_duplicate: boolean;
  missing_name: boolean;
}

export interface Agent {
  id: string;
  name: string;
  skills_dir: string;
  detected: boolean;
  enabled: boolean;
  auto_detected: boolean;
  linked_skills: string[];
}

export interface HistoryEntry {
  id: string;
  timestamp: string;
  action: string;
  skill_id: string;
  agent_id?: string;
  mode?: string;
  undone: boolean;
}

export interface AppConfig {
  version: number;
  sync_mode_default: string;
  agents: AgentConfig[];
  ui: {
    theme: string;
    locale: string;
  };
  history: {
    max_entries: number;
    snapshot_max_size_mb: number;
  };
}

export interface AgentConfig {
  id: string;
  name: string;
  skills_dir: string;
  enabled: boolean;
  auto_detected: boolean;
}

export type TabId = "overview" | "manage" | "history";

export type ConflictType = "SameNameDiffContent" | "DanglingLink";

export interface SkillIssue {
  skill_id: string;
  issue_type: ConflictType;
  description: string;
}

export interface DashboardData {
  agents: DashboardAgent[];
  shared_skills: SharedSkillInfo[];
  stats: DashboardStats;
}

export interface DashboardAgent {
  agent_id: string;
  agent_name: string;
  skill_count: number;
  skills: DashboardSkill[];
}

export interface DashboardSkill {
  skill_id: string;
  skill_name: string;
  shared_with: string[];
}

export interface SharedSkillInfo {
  skill_id: string;
  skill_name: string;
  agent_ids: string[];
}

export interface DashboardStats {
  total_skills: number;
  shared_count: number;
  per_agent_count: Record<string, number>;
}

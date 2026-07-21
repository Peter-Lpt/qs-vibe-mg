export interface SkillSource {
  from: string;
  source_kind?: "library" | "agent" | "project" | "external" | "marketplace";
  path: string;
  name: string;
  description: string;
  is_symlink: boolean;
  symlink_target?: string;
  content_hash: string;
  modified_at?: string;
  origin?: SkillOrigin;
  trust_level?: "explicit" | "inferred" | "unknown" | string;
  update_status?: "auto_update" | "best_effort" | "unknown" | string;
}

export interface SkillOrigin {
  method: string;
  provider?: string;
  url?: string;
  commit?: string;
  branch?: string;
  installed_at: string;
  installed_by?: string;
  trust_level?: string;
  source_path?: string;
  command?: string;
  update_command?: string;
  last_checked_at?: string;
}

export interface SkillUpdateCheck {
  skill_id: string;
  method: string;
  available: boolean;
  current_commit?: string;
  remote_commit?: string;
  checked_at: string;
  error?: string;
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
  from_plugin: boolean;
  plugin_source?: string;
}

export interface Agent {
  id: string;
  name: string;
  skills_dir: string;
  kind?: "agent" | "common" | "project" | "external";
  detect_dir?: string;
  additional_scan_dirs?: string[];
  tool_detected?: boolean;
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
  project_roots: string[];
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
  kind?: "agent" | "common" | "project" | "external";
  detect_dir?: string;
  additional_scan_dirs?: string[];
  enabled: boolean;
  auto_detected: boolean;
}

export interface ProjectRootSuggestion {
  path: string;
  is_current: boolean;
  matched_dirs: string[];
}

export type TabId = "manage" | "history";

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

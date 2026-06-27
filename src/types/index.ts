export interface SkillSource {
  from: string;
  path: string;
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

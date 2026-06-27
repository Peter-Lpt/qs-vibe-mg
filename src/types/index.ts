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
}

export interface Agent {
  id: string;
  name: string;
  skills_dir: string;
  detected: boolean;
  enabled: boolean;
}

// 全局组件类型声明，使 vue-tsc 识别 main.ts 注册的 lucide 图标组件。
import type {
  Network,
  List,
  ListTree,
  TreePine,
  GitBranch,
  FolderOpen,
  Folder,
  FolderClosed,
  FolderTree,
  Trash2,
  Link2,
  Link,
  Unlink,
  Link2Off,
  Package,
  Circle,
  CircleDashed,
  CircleAlert,
  TriangleAlert,
  Plus,
  Minus,
  RefreshCw,
  Eye,
  Copy,
  CopyPlus,
  Check,
  CheckCircle,
  X,
  XCircle,
  CircleSlash,
  ChevronRight,
  Search,
  FileBox,
  Box,
  Share2,
  ArrowLeftRight,
  GitMerge,
} from "@lucide/vue";

declare module "vue" {
  export interface GlobalComponents {
    Network: typeof Network;
    List: typeof List;
    ListTree: typeof ListTree;
    TreePine: typeof TreePine;
    GitBranch: typeof GitBranch;
    FolderOpen: typeof FolderOpen;
    Folder: typeof Folder;
    FolderClosed: typeof FolderClosed;
    FolderTree: typeof FolderTree;
    Trash2: typeof Trash2;
    Link2: typeof Link2;
    Link: typeof Link;
    Unlink: typeof Unlink;
    Link2Off: typeof Link2Off;
    Package: typeof Package;
    Circle: typeof Circle;
    CircleDashed: typeof CircleDashed;
    CircleAlert: typeof CircleAlert;
    TriangleAlert: typeof TriangleAlert;
    Plus: typeof Plus;
    Minus: typeof Minus;
    RefreshCw: typeof RefreshCw;
    Eye: typeof Eye;
    Copy: typeof Copy;
    CopyPlus: typeof CopyPlus;
    Check: typeof Check;
    CheckCircle: typeof CheckCircle;
    X: typeof X;
    XCircle: typeof XCircle;
    CircleSlash: typeof CircleSlash;
    ChevronRight: typeof ChevronRight;
    Search: typeof Search;
    FileBox: typeof FileBox;
    Box: typeof Box;
    Share2: typeof Share2;
    ArrowLeftRight: typeof ArrowLeftRight;
    GitMerge: typeof GitMerge;
  }
}

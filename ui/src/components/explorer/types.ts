import type {FileInfo} from "../../class.ts";

export type ExplorerEntryType = "folder" | "file";

export type ExplorerEntry = {
  type: ExplorerEntryType;
  name: string;
  path: string;
  modified: string;
  size?: number;
  extension?: string;
  file?: FileInfo;
}

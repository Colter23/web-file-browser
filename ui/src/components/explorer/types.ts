import type {FileInfo} from "../../class.ts";

export type DetailsColumnKey = "name" | "modified" | "type" | "size";

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

export type ExplorerDropAction = "copy" | "move";

export type ExplorerEntryPathDropPayload = {
  entries: ExplorerEntry[];
  target: {
    path: string;
    name: string;
  };
  action: ExplorerDropAction;
}

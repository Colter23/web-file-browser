import type {ArchiveFormat} from "../../class.ts";
import type {ExplorerEntry} from "../explorer/types.ts";

export type FileClipboardAction = "copy" | "cut";

export type OperationPanelKind = "createFile" | "createFolder" | "archive" | "extract";

export type OperationPanelState = {
  visible: boolean;
  kind: OperationPanelKind | null;
  title: string;
  message: string;
  primaryText: string;
  name: string;
  format: ArchiveFormat;
  entries: ExplorerEntry[];
  sourceEntry: ExplorerEntry | null;
  submitting: boolean;
}

export type DeleteConfirmState = {
  visible: boolean;
  entries: ExplorerEntry[];
  submitting: boolean;
  error: string;
}

export type PropertiesPanelState = {
  visible: boolean;
  entries: ExplorerEntry[];
}

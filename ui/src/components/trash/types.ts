import type {TrashRecord} from "../../class.ts";

export type TrashConfirmKind = "delete" | "empty";

export type TrashConfirmState = {
  visible: boolean;
  kind: TrashConfirmKind | null;
  records: TrashRecord[];
  submitting: boolean;
  error: string;
}

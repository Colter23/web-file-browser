import {onBeforeUnmount, onMounted} from "vue";
import type {Ref} from "vue";

type OutsidePointerDownOptions = {
  refs: Array<Ref<HTMLElement | null>>;
  enabled?: () => boolean;
  onOutsidePointerDown: (event: PointerEvent) => void;
}

export const useOutsidePointerDown = ({
  refs,
  enabled = () => true,
  onOutsidePointerDown
}: OutsidePointerDownOptions) => {
  const isInside = (target: EventTarget | null) => {
    if (!(target instanceof Node)) return false;
    return refs.some(ref => ref.value?.contains(target));
  }

  const handlePointerDown = (event: PointerEvent) => {
    if (!enabled() || isInside(event.target)) return;
    onOutsidePointerDown(event);
  }

  onMounted(() => {
    document.addEventListener("pointerdown", handlePointerDown, true);
  });

  onBeforeUnmount(() => {
    document.removeEventListener("pointerdown", handlePointerDown, true);
  });
}

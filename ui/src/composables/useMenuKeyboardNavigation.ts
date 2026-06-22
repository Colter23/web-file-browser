import {nextTick} from "vue";
import type {Ref} from "vue";

type MenuKeyboardNavigationOptions = {
  menuRef: Ref<HTMLElement | null>;
  buttonSelector?: string;
  onEscape?: () => void;
}

export const useMenuKeyboardNavigation = ({
  menuRef,
  buttonSelector = "button:not(:disabled)",
  onEscape
}: MenuKeyboardNavigationOptions) => {
  const menuButtons = () => {
    const menu = menuRef.value;
    if (!menu) return [];
    return Array.from(menu.querySelectorAll<HTMLButtonElement>(buttonSelector));
  }

  const focusMenuButton = (index: number) => {
    const buttons = menuButtons();
    if (!buttons.length) return;
    const nextIndex = (index + buttons.length) % buttons.length;
    buttons[nextIndex]?.focus({preventScroll: true});
  }

  const focusFirstMenuButton = async () => {
    await nextTick();
    focusMenuButton(0);
  }

  const moveMenuFocus = (direction: -1 | 1) => {
    const buttons = menuButtons();
    if (!buttons.length) return;
    const currentIndex = buttons.findIndex(button => button === document.activeElement);
    focusMenuButton(currentIndex < 0 ? 0 : currentIndex + direction);
  }

  const handleMenuKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      if (!onEscape) return false;
      event.preventDefault();
      event.stopPropagation();
      onEscape();
      return true;
    }
    if (event.key === "ArrowDown") {
      event.preventDefault();
      event.stopPropagation();
      moveMenuFocus(1);
      return true;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      event.stopPropagation();
      moveMenuFocus(-1);
      return true;
    }
    if (event.key === "Home") {
      event.preventDefault();
      event.stopPropagation();
      focusMenuButton(0);
      return true;
    }
    if (event.key === "End") {
      event.preventDefault();
      event.stopPropagation();
      focusMenuButton(menuButtons().length - 1);
      return true;
    }
    if (event.key === "Tab") {
      event.preventDefault();
      event.stopPropagation();
      moveMenuFocus(event.shiftKey ? -1 : 1);
      return true;
    }
    return false;
  }

  return {
    menuButtons,
    focusMenuButton,
    focusFirstMenuButton,
    moveMenuFocus,
    handleMenuKeyDown
  };
}

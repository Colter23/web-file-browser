const normalizeWheelDistance = (event: WheelEvent, delta: number, target: HTMLElement) => {
  if (event.deltaMode === 1) return delta * 40;
  if (event.deltaMode === 2) return delta * target.clientWidth;
  return delta;
}

export const scrollHorizontallyWithWheel = (event: WheelEvent) => {
  const target = event.currentTarget;
  if (!(target instanceof HTMLElement)) return;
  const delta = Math.abs(event.deltaX) > Math.abs(event.deltaY) ? event.deltaX : event.deltaY;
  if (!delta || target.scrollWidth <= target.clientWidth) return;
  const distance = normalizeWheelDistance(event, delta, target);
  event.preventDefault();
  target.scrollLeft += distance;
}

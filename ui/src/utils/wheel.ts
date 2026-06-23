export const scrollHorizontallyWithWheel = (event: WheelEvent) => {
  const target = event.currentTarget;
  if (!(target instanceof HTMLElement)) return;
  const delta = Math.abs(event.deltaX) > Math.abs(event.deltaY) ? event.deltaX : event.deltaY;
  if (!delta || target.scrollWidth <= target.clientWidth) return;
  event.preventDefault();
  target.scrollLeft += delta;
}

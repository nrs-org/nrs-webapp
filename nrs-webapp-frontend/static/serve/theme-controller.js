const dark = "night";
const light = "winter";

function getCurrentTheme() {
  return (
    localStorage.getItem("theme") ??
    (matchMedia("(prefers-color-scheme: dark)")?.matches ? dark : light)
  );
}

function onSwapBegin(swapElt) {
  // First, set the theme to the current theme (or default/user preference)
  document.documentElement.dataset.theme = getCurrentTheme();
}

function onSwapEnd(swapElt) {
  const input = swapElt.querySelector(".theme-controller");
  if (!input) return;

  const theme = getCurrentTheme();
  // Then, display the theme controller inputs and set their checked status to the
  // default theme
  input.parentElement.classList.remove("hidden");
  console.debug(input.parentElement);
  input.checked = theme === dark;
  input.addEventListener("change", () => {
    const theme = input.checked ? dark : light;
    localStorage.setItem("theme", theme);
  });
  // Finally, to make the theme controllers work, set the default theme to light
  document.documentElement.dataset.theme = light;
}

document.addEventListener("DOMContentLoaded", () => {
  onSwapBegin(document.documentElement);
  onSwapEnd(document.documentElement);
});

htmx.on("htmx:before:swap", (evt) => {
  onSwapBegin(evt.target);
});

htmx.on("htmx:after:swap", (evt) => {
  onSwapEnd(evt.target);
});

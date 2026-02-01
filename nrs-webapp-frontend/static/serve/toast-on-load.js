document
  .querySelectorAll(".nrs-toast")
  .forEach((t) => {
    setTimeout(() => t.querySelector(".close-button")?.click(), 10000),
  });

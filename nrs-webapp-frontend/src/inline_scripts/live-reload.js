const ev = new EventSource("/__watch");
let connectedOnce = false;

ev.onopen = () => {
  // This fires on first connect AND every reconnect after server restart
  if (connectedOnce) {
    console.log("Server restarted → reloading");
    location.reload();
  }
  connectedOnce = true;
};

// schedule a page reload on error
// (kinda unnecessary rn since this is meant to be used
// when database migrations were slow and is done in parallel
// with the server restart, but whatever))
if (document.documentElement.dataset.isError === "true") {
  setTimeout(() => location.reload(), 10000);
}

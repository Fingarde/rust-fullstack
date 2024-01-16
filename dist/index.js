const evtSource = new EventSource("__hot_reload");

evtSource.onmessage = function (event) {
    console.log(event.data);
    if (event.data === "reload") {
        location.reload(true);
    }
}

(function () {
  document.addEventListener("DOMContentLoaded", function () {
    console.log("I loaded");
    const socket = new WebSocket("ws://localhost:4040/ws");

    socket.addEventListener("open", function (event) {
      console.info("ws open");
      socket.send("HI, SERVER!");
    });

    socket.addEventListener("message", function (event) {
      console.log("got message: ", event.data);
    });

    socket.addEventListener("close", function () {
      console.info("ws closed");
    });
  });
})();

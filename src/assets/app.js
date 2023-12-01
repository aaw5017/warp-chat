(function () {
  document.addEventListener("DOMContentLoaded", function () {
    console.log("I loaded");
    const socket = new WebSocket("ws://localhost:4040/ws");

    socket.addEventListener("open", function (event) {
      console.info("ws open");
    });

    socket.addEventListener("message", function (event) {
      console.log("got message: ", event.data);
    });

    socket.addEventListener("close", function () {
      console.info("ws closed");
    });

    const btn = document.getElementById("send-btn");
    const input = document.getElementById("message-input");
    btn.addEventListener("click", function () {
      input.value && socket.send(input.value);
    });
  });
})();

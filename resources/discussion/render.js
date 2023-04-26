function fetch_speaking_order() {
  fetch(window.location.href + "/speaking_order.html")
      .then(res => res.text())
      .then(data => document.getElementById("speaking_order").innerHTML = data);
}

function show_master_controls() {
  if (confirm("Are you sure you would like to display the master controls? Please only do this if you are the speaker")) {
    document.getElementById("master_controls").attributes.removeNamedItem("hidden");
    document.getElementById("show_master_controls").setAttribute("hidden", "");
  }
}

window.onload = (event) => {
  fetch_speaking_order();
  setInterval(fetch_speaking_order, 1000);
};
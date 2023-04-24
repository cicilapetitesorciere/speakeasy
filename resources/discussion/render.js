function fetch_speaking_order() {
  fetch(window.location.href + "/speaking_order.html")
      .then(res => res.text())
      .then(data => document.getElementById("speaking_order").innerHTML = data);
}

window.onload = (event) => {
  fetch_speaking_order();
  setInterval(fetch_speaking_order, 1000);
};
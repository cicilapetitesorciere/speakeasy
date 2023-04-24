async function fetch_speaking_order() {
  fetch(window.location.href + "/speaking_order.html")
      .then(res => res.text())
      .then(data => document.getElementById("speaking_order").innerHTML = data);
  setTimeout(() => { fetch_speaking_order(); }, 1000);
}

window.onload = fetch_speaking_order();
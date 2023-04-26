function refresh() {
  fetch(window.location.href + "/status")
      .then(res => res.text())
      .then(data => {
        const parsed_data = JSON.parse(data);
        if (parsed_data.status == "Normal") {
          document.getElementById("speaking_order").innerHTML = parsed_data.speaking_order;
          document.getElementById("header").innerHTML = "Speakeasy - " + parsed_data.duration;
          document.getElementById("controls").removeAttribute("hidden");
          document.getElementById("point_of_order").setAttribute("hidden","");
        } else if (parsed_data.status == "Paused") {
          document.getElementById("speaking_order").innerHTML = "";
          document.getElementById("controls").setAttribute("hidden","");
          document.getElementById("point_of_order").removeAttribute("hidden");
        }
          
      });
}

function show_master_controls() {
  if (confirm("Are you sure you would like to display the master controls? Please only do this if you are the speaker")) {
    document.getElementById("master_controls").attributes.removeNamedItem("hidden");
    document.getElementById("show_master_controls").setAttribute("hidden", "");
  }
}

window.onload = (event) => {
  refresh();
  setInterval(refresh, 1000);
};
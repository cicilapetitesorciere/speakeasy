function joinDiscussion() {
    fetch(window.location.href + "/discussion/" + document.getElementById("disc_id").value + "/speaking_order.html")
      .then(res => res.text())
      .then(data => {
        if (data != "" || confirm("No discussion with ID \"" + document.getElementById("disc_id").value + "\". Would you like to create it?")) {
            window.location.href = "/discussion/" + document.getElementById("disc_id").value;
        }
      });
}
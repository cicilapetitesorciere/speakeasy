function addSpeech(stype) {
  const name = document.getElementById("speaker_name").value;
  fetch(window.location.href + "/add_speaker", {
    method: "POST",
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      name: name, 
      stype: stype
    }),
  });
  fetch_speaking_order();
}

function next() {
  fetch(window.location.href + "/next", {method: "POST"});
  fetch_speaking_order();
}

function previous() {
  fetch(window.location.href + "/previous", {method: "POST"});
  fetch_speaking_order();
}
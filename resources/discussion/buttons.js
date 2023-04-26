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
  refresh();
}

function pointOfOrder() {
  fetch(window.location.href + "/setpause/pause", {
    method: "POST",
  });
  refresh();
}

function resolvePointOfOrder() {
  fetch(window.location.href + "/setpause/unpause", {
    method: "POST",
  });
  refresh();
}

function next() {
  fetch(window.location.href + "/next", {method: "POST"});
  refresh();
}

function previous() {
  fetch(window.location.href + "/previous", {method: "POST"});
  refresh();
}

function change_priority_mode(option) {
  fetch(window.location.href + "/set_priority_mode/" + option.value, {method: "POST"});
  refresh();
}
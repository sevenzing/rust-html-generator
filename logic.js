var inputs = document.getElementsByName("file");
var content = document.getElementById("code");

for (let i = 0; i < inputs.length; i++) {
    inputs[i].addEventListener("input", (e) => {
        let filename = e.target.value;
        content.innerHTML = document.getElementById(filename).innerHTML;
    })
}

// function addLineClass (pre) {
//     var lines = pre.innerText.split("\n"); // can use innerHTML also
//     while(pre.childNodes.length > 0) {
//         pre.removeChild(pre.childNodes[0]);
//     }
//     for(var i = 0; i < lines.length; i++) {
//         var span = document.createElement("span");
//         span.className = "line";
//         span.innerText = lines[i]; // can use innerHTML also
//         pre.appendChild(span);
//         pre.appendChild(document.createTextNode("\n"));
//     }
// }
// window.addEventListener("load", function () {
//     var pres = document.getElementsByTagName("pre");
//     for (var i = 0; i < pres.length; i++) {
//         addLineClass(pres[i]);
//     }
// }, false);

var inputs = document.getElementsByName("file");
var content = document.getElementById("code");

for (let i = 0; i < inputs.length; i++) {
    inputs[i].addEventListener("input", (e) => {
        let filename = e.target.value;
        content.innerHTML = document.getElementById(filename).innerHTML;
    })
}


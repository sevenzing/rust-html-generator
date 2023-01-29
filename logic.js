var inputs = document.getElementsByName("file");
var content = document.getElementById("code");
var current_file = null;

const update = () => {
    console.log('UPDATE')
    const params = new URLSearchParams(document.location.search);
    let filename = params.get('filename');
    selectFileWithName(filename)

    let line_no = document.location.hash.replace('#L', '');
    let id = 'LC' + line_no;
    let line = document.getElementById(id);
    content.querySelectorAll('.line-content').forEach(l => l.classList.remove('line-selected'))
    line.classList.add('line-selected')
}

const selectFileWithName = (filename) => {
    if (current_file !== filename) {
        current_file = filename
        let file_content = document.getElementById(filename);
        if (file_content != null) {
            content.innerHTML = file_content.innerHTML;
        }
    }
}

const treeClicked = (e) => {
    let filename = e.target.value;
    const params = new URLSearchParams(document.location.search);
    params.set('filename', filename)
    let url = document.location.pathname + '?' + params.toString() + document.location.hash
    window.history.pushState({}, null, url);
    selectFileWithName(filename);
    update();
}

for (let i = 0; i < inputs.length; i++) {
    inputs[i].addEventListener("input", treeClicked)
}

window.addEventListener('locationchange', () => {
    update()
})

window.onload = update
window.onhashchange = update

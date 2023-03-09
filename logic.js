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
    if (line) {
        line.classList.add('line-selected')
        line.scrollIntoView({ behavior: "smooth", block: "end", inline: "nearest" });
    }
}

const selectFileWithName = (filename) => {
    if (current_file !== filename) {
        current_file = filename
        let file_content = document.getElementById(filename);
        if (file_content != null) {
            content.innerHTML = file_content.innerHTML;
            onFileChanged()
        }
    }
}

const treeClick = (filename) => {
    document.querySelector(`input[value='${filename}']`).click()
}

const treeClicked = (e) => {
    let filename = e.target.value;
    jumpTo(filename, null)
}

for (let i = 0; i < inputs.length; i++) {
    inputs[i].addEventListener("input", treeClicked)
}

window.addEventListener('locationchange', () => {
    update()
})


const jumpTo = (filename, line_no) => {
    console.log("jump to", filename, line_no);
    const params = new URLSearchParams(document.location.search);
    if (filename) {
        params.set('filename', filename)
    }
    let hash = document.location.hash;
    if (line_no) {
        hash = `#L${line_no}`
    } else {
        hash = ''
    }

    let url = document.location.pathname + '?' + params.toString() + hash;
    window.history.pushState({}, null, url);
    selectFileWithName(filename);
    update();
}

// Jumps
const initializeJumps = () => {
    document.querySelectorAll('.jump').forEach(j => {
        const jump_data = JSON.parse(j.getAttribute('jump-data').replaceAll("'", '"'));
        j.onclick = function() {
            if (pressedKeys[META_KEY]) {
                treeClick(jump_data['file'])
                jumpTo(jump_data['file'], jump_data['location']['start']['line'])
            }
        }
    });
}

const onFileChanged = () => {
    initializeJumps()
}

const handleMetaUp = () => {
    document.querySelectorAll('.jump').forEach(j => {
        j.classList.remove('jump-hover')
    })
}
const handleMetaDown = () => {
    document.querySelectorAll('.jump').forEach(j => {
        j.classList.add('jump-hover')
    })
}


const main = () => {
    update()
    initializeJumps()
}

const META_KEY = 91;
var pressedKeys = {};

window.onload = main
window.onhashchange = update
window.onkeyup = function(e) { 
    pressedKeys[e.keyCode] = false; 
    if (e.keyCode == META_KEY) handleMetaUp();
}
window.onkeydown = function(e) { 
    pressedKeys[e.keyCode] = true;
    if (e.keyCode == META_KEY) handleMetaDown();
}



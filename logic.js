var inputs = document.getElementsByName("file");
var content = document.getElementById("code");
var current_file = null;

var lockChanging = false;

const update = () => {
    console.log('UPDATE')
    const params = new URLSearchParams(document.location.search);
    let filename = params.get('filename');
    selectFileWithName(filename)
    treeClick(filename)

    let line_no = document.location.hash.replace('#L', '');
    let line_content = document.getElementById('LC' + line_no);
    let line = document.getElementById('L' + line_no)
    content.querySelectorAll('.line-content').forEach(l => l.classList.remove('line-selected'))
    if (line_content) {
        line_content.classList.add('line-selected')
        line.scrollIntoView({ behavior: "smooth", block: "center", inline: "nearest" });
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
    if (!lockChanging) showFile(filename)
}

for (let i = 0; i < inputs.length; i++) {
    inputs[i].addEventListener("input", treeClicked)
}

window.addEventListener('locationchange', () => {
    update()
})


const buildHrefFromJump = (filename, line_no) => {
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
    return url
}

const jumpTo = (jump_data, pushHistory = false) => {
    console.log('jump to', jump_data)
    let from_url = buildHrefFromJump(jump_data['from']['file'], jump_data['from']['location']['start']['line']);
    let to_url = buildHrefFromJump(jump_data['to']['file'], jump_data['to']['location']['start']['line']);

    if (pushHistory) {
        console.log('l: ', window.history.length)
        pushHistoryStateSafe(from_url, window.location.href);
        pushHistoryStateSafe(to_url, from_url);
        console.log('l: ', window.history.length)
    }
    selectFileWithName(jump_data['from']['file']);
    update();
}

const pushHistoryStateSafe = (href, prevHref) => {
    if (window.location.href != href) {
        console.log('push', href)
        window.history.pushState({prevUrl: prevHref}, null, href);
    } else {
        console.log('history duplicate, ignore')
    }
}

const replaceCurrentState = (href) => {
    window.history.replaceState(window.history.state, null, href);
}

const handleBackButton = () => {
    lockChanging = true;
    update();
    lockChanging = false;
}

const showFile = (filename) => {
    let url = buildHrefFromJump(filename, null);
    replaceCurrentState(url);
    selectFileWithName(filename);
    update();
}

// Jumps
const initializeJumps = () => {
    document.querySelectorAll('.jump').forEach(j => {
        const jump_data = JSON.parse(j.getAttribute('jump-data').replaceAll("'", '"'));
        j.onclick = function() {
            if (pressedKeys[META_KEY]) {
                treeClick(jump_data['to']['file'])
                jumpTo(jump_data, true)
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

window.onpopstate = handleBackButton


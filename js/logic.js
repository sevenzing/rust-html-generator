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
            changeHeaderFilename(filename);
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

const jumpTo = (jumpDest, fromDest, pushHistory = false) => {
    console.log('jump to', jumpDest)
    let from_url = buildHrefFromJump(fromDest['file'], fromDest['loc']['line']);
    let to_url = buildHrefFromJump(jumpDest['file'], jumpDest['loc']['line']);

    if (pushHistory) {
        pushHistoryStateSafe(from_url, window.location.href);
        pushHistoryStateSafe(to_url, from_url);
    }
    selectFileWithName(jumpDest['file']);
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

const changeHeaderFilename = (filename) => {
    const badge = document.querySelector('.filename > .badge')
    if (filename) {
        badge.style.display = ''
        badge.innerHTML = filename
    } else {
        badge.style.display = 'none'
    }
}

// Jumps
const initializeJumps = () => {
    document
        .querySelectorAll('.code-section .jump')
        .forEach(jump => {
            if (!jump.classList.contains('jumpmenu')) {
                const jump_data = JSON.parse(jump.getAttribute('jump-data').replaceAll("'", '"'));
                var menu = document.createElement('div');
                menu.innerHTML = buildInnerHTMLForJump(jump_data);
                jump.appendChild(menu.lastChild)
                jump.classList.add('jumpmenu')
            }
            // const jump_data = JSON.parse(jump.getAttribute('jump-data').replaceAll("'", '"'));
            // jump.onclick = function() {
            //     if (pressedKeys[META_KEY]) {
            //         treeClick(jump_data['def']['file'])
            //         jumpTo(jump_data, true)
            //     }
            // }
        }
    );
    initializeJumpsMenu();
    initializeJumpButtons();
}

const buildInnerHTMLForJump = (jump_data) => {
    const def = renderButton(jump_data['def'])
    const refs = jump_data['refs'].map((ref) => renderButton(ref)).join('\n')

    return `<div class="jump__content jump__content--below">
        <div class="tab-container">
            <div class="tab-headers">
                <div class="tab-header active definitions">Definitions</div>
                <div class="tab-header references">References</div>
            </div>

            <div class="definitions tab-content">
                ${def}
            </div>
            <div class="references tab-content hide">
                ${refs}
            </div>
        </div>
    </div>`
}

const renderButton = (jumpDest) => {
    const f = jumpDest['file'];
    const l = jumpDest['loc']['line'];
    return `<div class="row jump-button" jump_file='${f}' jump_line='${l}'>${f}:${l}</div>`
}

const onFileChanged = () => {
    initializeJumps();
    initializeFolds();
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

// folding
const initializeFolds = () => {
    document.querySelectorAll('.code-section .line-fold').forEach(fold => {
        const startLine = Number(fold.getAttribute('data-fold-start-line'));
        const endLine = Number(fold.getAttribute('data-fold-end-line'));
        fold.onclick = () => {
            const mainContentLine = document.querySelectorAll(`.code-section #LC${startLine}`)[0];
            const relatedLines = Array
                .from(document.querySelectorAll(".code-section .table-line"))
                .filter(el => {
                    const n = el.getAttribute('number');
                    return startLine < n && n < endLine
                })

            const toClose = fold.classList.contains('arrow--right');
            if (toClose) {
                closeLines(relatedLines, fold, mainContentLine)
            } else {
                openLines(relatedLines, fold, mainContentLine)
            }
        }
    });
}

const closeLines = (lines, fold, mainLine) => {
    fold.classList.remove('arrow--right');
    fold.classList.add('arrow--down');
    mainLine.classList.add('line-folded')
    lines.forEach(line => {
        line.style.display = 'none'
    })
}

const openLines = (lines, fold, mainContentLine) => {
    fold.classList.remove('arrow--down');
    fold.classList.add('arrow--right');
    mainContentLine.classList.remove('line-folded')
    lines.forEach(line => {
        line.style.display = ''
    })
}

const initializeResize = () => {
    var resize = document.querySelector("#resize");
    var tree = document.querySelector(".tree");
    var left = document.querySelector(".left");
    var content = document.querySelector(".content");
    var moveX = tree.getBoundingClientRect().width + 
                resize.getBoundingClientRect().width / 2;

    var drag = false;
    resize.addEventListener("mousedown", function (e) {
        drag = true;
        moveX = e.x;
    });

    content.addEventListener("mousemove", function (e) {
    moveX = e.x;
    if (drag)
        tree.style.width =
            moveX - resize.getBoundingClientRect().width / 2 + "px";
        left.style.minWidth = tree.style.width;
    });

    content.addEventListener("mouseup", function (e) {
        drag = false;
    });
}

const initializeJumpsMenu = () => {
    const openClassName = 'is-open';
    const jumpmenus = document.querySelectorAll('.code-section span.jump:not(.jumpmenu__content)');
    const body = document.querySelector('body');
    jumpmenus.forEach(function (jumpmenu) {
        const onClick = (e) => {
            e.stopPropagation();
            if (e.target.classList.contains('jump') && e.target.classList.contains('jump-hover')) {
                jumpmenu.classList.add(openClassName); 
            }
        }
        jumpmenu.addEventListener('click', onClick);
    });

    body.addEventListener('click', function (e) {
        if (!e.target.classList.contains('jump')) {
            const openJumpmenus = document.querySelectorAll('.jump.' + openClassName);
            openJumpmenus.forEach(function (jumpmenu) {
                jumpmenu.classList.remove(openClassName);
            });
        }
    });

    document.querySelectorAll('.code-section .tab-container').forEach((tab) => {
        const def_header = tab.querySelector('.tab-header.definitions');
        const ref_header = tab.querySelector('.tab-header.references');

        def_header.addEventListener('click', (e) => {
            tab.querySelector('.tab-content.references').classList.add('hide')
            tab.querySelector('.tab-content.definitions').classList.remove('hide')
            ref_header.classList.remove('active')
            def_header.classList.add('active')
        })

        tab.querySelector('.tab-header.references').addEventListener('click', (e) => {
            tab.querySelector('.tab-content.definitions').classList.add('hide')
            tab.querySelector('.tab-content.references').classList.remove('hide')
            def_header.classList.remove('active')
            ref_header.classList.add('active')
        })
    })
}

const initializeJumpButtons = () => {
    document.querySelectorAll('.code-section .jump-button').forEach((btn) => {
        btn.onclick = () => {
            const jump = btn.closest('.jump');
            const jump_data = JSON.parse(jump.getAttribute('jump-data').replaceAll("'", '"'));
            const jump_file = btn.getAttribute('jump_file');
            const jump_line = btn.getAttribute('jump_line');
            treeClick(jump_file)
            jumpTo({
                file: jump_file,
                loc: {
                    line: jump_line
                },
            }, jump_data['from'], true)

            }
    })
}

const main = () => {
    initializeResize();
    update();
    onFileChanged();
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


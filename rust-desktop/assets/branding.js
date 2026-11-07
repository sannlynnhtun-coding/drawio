(() => {
    function brand() {
        if (window.Editor) Editor.prototype.appName = 'Drawio';
        const title = document.title.replace(/draw\.io(?: app)?/gi, 'Drawio')
            .replace('Flowchart Maker & Online Diagram Software', 'Drawio');
        if (document.title !== title) document.title = title;
        for (const icon of document.querySelectorAll('.geMenubar img[title="draw.io"], .geMenubar img[alt="draw.io"]')) {
            icon.src = '/images/drawlogo256.png';
            icon.alt = 'Drawio';
            icon.title = 'Drawio';
        }
    }
    addEventListener('DOMContentLoaded', () => {
        brand();
        new MutationObserver(brand).observe(document.documentElement, {childList:true, subtree:true});
    }, {once:true});
})();

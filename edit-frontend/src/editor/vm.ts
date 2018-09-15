/*
SeekForward(n),
// Error when cursor exceeds either boundary more than [0, len] inclusive
// Counts elements, including whole spans.

Enter() 
Unenter()

DeleteElements(n) // Deletes elements
InsertDocString(DocString),

UnwrapSelf()
WrapPrevious(n, Attrs)

*/

function assert(condition: boolean) {
    if (!condition) {
        throw new Error('Condition failed.');
    }
}


export function vm(el: Node) {
    let stack: Array<[number, Node]> = [[0, el]];

    let cur = () => { return stack[stack.length - 1]; };
    let curNode = () => {
        assert(cur()[0] <= cur()[1].childNodes.length);
        return cur()[1].childNodes[cur()[0]];
    };

    return {
    		is_done() {
					return (stack.length == 1 && cur()[0] >= cur()[1].childNodes.length) || stack.length == 0;
        },

        Enter() {
            let node = curNode();
            assert(node.nodeType == 1);
            stack.push([0, node]);
        },
        Unenter() {
            assert(stack.length > 1); // Can't unenter root
            stack.pop();
            
            // Also perform stack advancement
            cur()[0] += 1;
            assert(cur()[1].childNodes.length <= cur()[0]);
        },
        AdvanceElements(n: number) { // By element
            cur()[0] += n;
            assert(cur()[1].childNodes.length <= cur()[0]);
        },
        DeleteElements(n: number) {
            for (let i = 0; i < n; i++) {
                assert(curNode() !== null);
                curNode().parentNode!.removeChild(curNode());
            }
        },
        InsertDocString(s: string) {
            let span = document.createElement('span');
            span.innerText = s;
            cur()[1].insertBefore(span, curNode());
        },
        WrapPrevious(n: number, attrs: any) {
            let div = document.createElement('div');
            Object.keys(attrs).forEach(key => {
            	div.setAttribute(key, attrs[key]);
            })
            // TODO attrs
            cur()[1].insertBefore(div, curNode());
            for (let i = 0; i < n; i++) {
                div.insertBefore(div.previousSibling!, div.firstChild);
            }
        },
        UnwrapSelf() {
            let node = cur()[1];
		        stack.pop();
            while (node.childNodes.length) {
            		cur()[0] += 1;
                node.parentNode!.insertBefore(node.firstChild!, node);
            }
            node.parentNode!.removeChild(node);
        }
    }
}

/*
let root = document.querySelector('#app');
let B = vm(root);
let program = [
	['Enter'],
  ['DeleteElements', 1],
  ['InsertDocString', 'Goodbye, '],
  ['UnwrapSelf'],
  ['WrapPrevious', 2, {'class': 'cool'}],
];
program.forEach(item => {
	B[item[0]].apply(B, item.slice(1));
})
assert(B.is_done());
console.log('Done');
*/
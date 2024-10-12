const messageCache = {};
let lockedScroll = true;

function displayMessages(messages) {
    const textarea = document.getElementById('messages');
    textarea.textContent = messages.map(message => `${message.author}> ${message.message}`).join('\n');

    if (lockedScroll) {
        textarea.scrollTop = textarea.scrollHeight;
    }
}

function mergeAndRenderMessages(messages) {
    messages.map(message => messageCache[message.id] = message);

    displayMessages(Object.entries(messageCache).map(([_id, message]) => message));
}

function sendMessage(message, successCallback) {
    fetch('/messages', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: message,
    }).then(response => response.json())
        .then(data => {
            console.log('Success:', data);
            successCallback(data);
        })
        .catch((error) => {
            console.error('Error:', error);
        });
}

function refreshMessages() {
    fetch('/messages')
        .then(response => response.json())
        .then(data => {
            console.log('Success:', data);
            mergeAndRenderMessages(data);
        })
        .catch((error) => {
            console.error('Error:', error);
        });
}

(function() {
    const chatBar = document.getElementById('chatbar');
    const textarea = document.getElementById('messages');

    chatBar.onsubmit = (event) => {
        event.preventDefault();
        const input = chatBar.querySelector('input');

        if (!input.value) {
            return false;
        }

        sendMessage(input.value, (data) => {
            input.value = '';
            mergeAndRenderMessages(data);
        });

        return false;
    };

    textarea.onscroll = () => {
        lockedScroll = textarea.scrollTop + textarea.clientHeight >= textarea.scrollHeight;
    };

    refreshMessages();
    setInterval(() => {
        refreshMessages();
    }, 5000);
})()
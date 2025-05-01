using UnityEngine;
using NativeWebSocket;
using TMPro;

public class WebSocketClient : MonoBehaviour
{
    WebSocket websocket;

    public TMP_InputField inputField;
    public TextMeshProUGUI outputText;

    async void Start()
    {
        websocket = new WebSocket("ws://localhost:8080/ws");

        websocket.OnOpen += () =>
        {
            Debug.Log("✅ Connected to WSS server");
        };

        websocket.OnMessage += (bytes) =>
        {
            string message = System.Text.Encoding.UTF8.GetString(bytes);
            Debug.Log("📩 Received: " + message);
            outputText.text = message;
        };

        websocket.OnError += (e) =>
        {
            Debug.Log("❌ Error: " + e);
        };

        websocket.OnClose += (e) =>
        {
            Debug.Log("🔌 Connection closed");
        };

        await websocket.Connect();
    }

    public async void SendMessage()
    {
        if (websocket.State == WebSocketState.Open)
        {
            string msg = inputField.text;
            await websocket.SendText(msg);
            Debug.Log("📤 Sent: " + msg);
        }
    }

    void Update()
    {
        websocket?.DispatchMessageQueue();
    }

    private async void OnApplicationQuit()
    {
        if (websocket != null && websocket.State == WebSocketState.Open)
        {
            await websocket.Close();
        }
    }
}

# ACI-CN-RPCN-Public-API 文档  
## 基本信息  
### 协议类型：WebSocket  
支持binary和text消息，API始终以text消息响应，如果使用binary消息，确保以`UTF-8`编码  
### 地址URL：`ws://{PublicAPIHost}:{PublicAPIPort}`  
`PublicAPIHost` 和 `PublicAPIPort` 在 `rpcn.cfg` 内设置，任一留空时表示不启用API服务
## 请求基本格式  
```
{
    "apiName": "ACI-CN-Public-API",
	"apiVersion": "1.0",
	"requestID": "MyIDWithLessThan64Characters",
	"messageType": "APIStateRequest",
    "data": {
        "requestParam1": {}
    }
}
```
`apiName`：固定为 `ACI-CN-Public-API`，如果该参数不正确则不作响应  
`apiVersion`：该 API 版本将在行为/负载发生不兼容更改之前保持不变。这意味着可能会向 API 添加新功能（包括现有负载中的新字段），而无需增加版本号。客户端应保证能处理这种情况，并且在反序列化时遇到未知字段不会崩溃。  
`requestID`：可以在每个请求中添加 `requestID` 字段。这是可选的，但建议添加，因为它允许客户端识别对请求的响应。该 ID 还将用于在客户端日志中记录请求及任何错误。如果出现任何问题，可以使用此 ID 作为参考，在日志中检查与该请求相关的任何错误。可以为每个请求使用相同的 ID 或不同的 ID。如果提供，ID 应仅包含 ASCII 字符，长度至少为 1 且最多为 64 个字符。 如果未添加 `requestID` 字段，服务端将为请求生成一个随机的 UUID，并随响应返回。  
`messageType`：需要请求的功能或方法对应的请求体名称  
`data`：可选，请求参数或额外数据。
## 响应基本格式  
```
{
	"apiName": "ACI-CN-Public-API",
	"apiVersion": "1.0",
	"timestamp": 1782298143358,
	"messageType": "APIStateResponse",
	"requestID": "MyIDWithLessThan64Characters",
	"data": {
		"serverAPIVersion": "1.3",
		"currentSessionAuthenticated": false
	},
    "error": null
}
```
`apiName`：同响应  
`apiVersion`：同响应，但服务端在这里应该附上自己当前的API版本  
`timestamp`：服务器响应时的时间戳，毫秒级  
`messageType`：需要请求的功能或方法对应的响应体名称  
`data`：响应参数或额外数据  
## 目前的API接口  
## APIState
### APIStateRequest  
```
{
    "apiName": "ACI-CN-Public-API",
	"apiVersion": "1.0",
	"requestID": "MyIDWithLessThan64Characters",
	"messageType": "APIStateRequest"
}
```
### APIStateResponse
```
{
	"apiName": "ACI-CN-Public-API",
	"apiVersion": "1.0",
	"timestamp": 1782298143358,
	"messageType": "APIStateResponse",
	"requestID": "MyIDWithLessThan64Characters",
	"data": {
		"serverAPIVersion": "1.0"
	},
    "error": null
}
```
`serverAPIVersion`：当前服务端的API版本  

## GetRoomList
### GetRoomListRequest
```
{
    "apiName": "ACI-CN-Public-API",
	"apiVersion": "1.0",
	"requestID": "MyIDWithLessThan64Characters",
	"messageType": "GetRoomListRequest",
    "data": {
        "communicationID": "NPWR04428_00"
    }
}
```
`communicationID`：可为null，为null时，返回所有 `communicationID` 上的所有房间
### GetRoomListResponse  
```
{
	"apiName": "ACI-CN-Public-API",
	"apiVersion": "1.0",
	"timestamp": 1782298143358,
	"messageType": "GetRoomListResponse",
	"requestID": "MyIDWithLessThan64Characters",
	"data": {
        "NPWR04428_00": {
            "playerCount"：1,
		    "roomList": [
                {
                    "currentPlayerCount": 1,
                    "maxPlayerCount": 8,
					"hostPlayerName": "playerName",
                    "roomInfo": {
                        "roomDataExternal": {},
                        "roomDataInternal": {}
                    }
                }
            ]
        }
	},
    "error": null
}
```
`data` 内是以 `communicationID`为key的房间对象  
`playerCount`：当前 `communicationID` 的在线玩家总人数  
`roomList`：房间列表  
`currentPlayerCount`：当前房间玩家人数  
`maxPlayerCount`：当前房间最大玩家人数  
`hostPlayerName`：房主的RPCN名称  
`roomInfo`：房间属性，未定
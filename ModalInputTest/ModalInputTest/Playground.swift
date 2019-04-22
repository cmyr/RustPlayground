//
//  Playground.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-04-22.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

/// This provides a swift-y wrapper around the rust playground library.

import Foundation

struct PlaygroundError {
    let message: String
}

extension PlaygroundError: Error {}

func listToolchains() -> Result<[Toolchain], PlaygroundError> {
    let response = JsonResponse.from(externFn: playgroundGetToolchains)
    switch response {
    case .ok(let result):
        let data = (result as! [AnyObject])
        let toolchains = data.map { Toolchain.fromJson(json: $0)! }
        return .success(toolchains)
    case .error(let errorString):
        return .failure(PlaygroundError(message: errorString))
    }
}

struct Toolchain {
    let name: String
    let channel: String
    let date: String?

    static func fromJson(json: AnyObject) -> Toolchain? {
        if let json = json as? [String: AnyObject] {
            let name = json["name"] as! String
            let channel = json["channel"] as! String
            let date = json["date"] as? String
            return Toolchain(name: name, channel: channel, date: date)
        }
        return nil
    }
}

enum JsonResponse {
    case error(String)
    case ok(Any)

    /// Wrapper around an external function that returns json. Handles basic
    /// parsing and freeing memory.
    ///
    /// we control all the messages we send and receive, so we assume all
    /// messages are well-formed.
    static func from(externFn: () -> UnsafePointer<Int8>?) -> JsonResponse {
        let cString = externFn()!
        defer { playgroundStringFree(cString) }

        let string = String(cString: cString, encoding: .utf8)!
        let message = try! JSONSerialization.jsonObject(with: string.data(using: .utf8)!) as! [String: AnyObject]
        if let result = message["result"] {
            return .ok(result)
        } else if let error = message["error"] {
            return .error(error as! String)
        } else {
            fatalError("invalid json response: \(message)")
        }
    }
}


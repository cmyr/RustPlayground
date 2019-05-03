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
    let code: Int

    init?(json: [String: AnyObject]) {
        if let message = json["message"] as? String,
            let code = json["code"] as? Int {
            self.message = message
            self.code = code
        } else {
            return nil
        }
    }
}

extension PlaygroundError: Error {}

func listToolchains() -> Result<[Toolchain], PlaygroundError> {
    let response = JsonResponse.from(externFn: playgroundGetToolchains)
    switch response {
    case .ok(let result):
        let data = (result as! [AnyObject])
        let toolchains = data.map { Toolchain.fromJson(json: $0)! }
        return .success(toolchains)
    case .error(let error):
        return .failure(error)
    }
}

func executeTask(inDirectory buildDir: URL, task: CompilerTask, stderr: @escaping ((String) -> ())) -> Result<CompilerResult, PlaygroundError> {
    GlobalTaskContext.stderrCallback = stderr
    let buildPath = buildDir.path
    let taskJson = task.toJson()
    let response = JsonResponse.from { playgroundExecuteTask(buildPath, taskJson, stderrCallback) }

    switch response {
    case .ok(let result):
        let data = result as! [String: AnyObject]
        return .success(CompilerResult.fromJson(data))
    case .error(let err):
        return .failure(err)
    }
}

/// The least bad way I could think of to pipe callbacks from rust back
/// to whoever cares about them for this action
fileprivate class GlobalTaskContext {
    static var stderrCallback: ((String) -> ())? = nil
}

/// Called over the FFI boundry with lines from stderr
func stderrCallback(linePtr: UnsafePointer<Int8>?) {
    if let ptr = linePtr {
        let line = String(cString: ptr)
        if let callback = GlobalTaskContext.stderrCallback {
            callback(line)
        } else {
            print("NO STDERR callback for line: '\(line)'")
        }
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

    /// The name of this toolchain, suitable for display in menus etc
    var displayName: String {
        var base = channel.capitalized
        if let date = self.date {
            base = base + " (\(date))"
        }
        return base
    }
}
// TODO: this can probably just be a Result<T, E>
enum JsonResponse {
    case error(PlaygroundError)
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
            let error = error as! [String: AnyObject]
            return .error(PlaygroundError(json: error)!)
        } else {
            fatalError("invalid json response: \(message)")
        }
    }
}


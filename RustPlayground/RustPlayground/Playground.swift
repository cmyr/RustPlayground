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
    let code: Int32
}

extension PlaygroundError: Error {}

func listToolchains() -> Result<[Toolchain], PlaygroundError> {
    let response = PlaygroundResult.from { err in playgroundGetToolchains(err) }
    return response.map({ (toolchains) -> [Toolchain] in
        let data = toolchains as! [AnyObject]
        return data.map { Toolchain.fromJson(json: $0)! }
    })
}

func executeTask(inDirectory buildDir: URL, task: CompilerTask, stderr: @escaping ((String) -> ())) -> Result<CompilerResult, PlaygroundError> {
    GlobalTaskContext.stderrCallback = stderr
    let buildPath = buildDir.path
    let taskJson = task.toJson()
    let response = PlaygroundResult.from { err in playgroundExecuteTask(buildPath, taskJson, stderrCallback, err) }
    return response.map { CompilerResult.fromJson($0 as! [String: AnyObject]) }
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

typealias PlaygroundResult = Result<Any, PlaygroundError>

extension PlaygroundResult {
    /// Call an external function, doing error handling and json parsing.
    static func from(externFn: (UnsafeMutablePointer<ExternError>) -> UnsafePointer<Int8>?) -> PlaygroundResult {
        var error = ExternError()
        let result = externFn(&error);

        if error.code != 0 {
            let message = String(cString: error.message, encoding: .utf8)!
            let error = PlaygroundError(message: message, code: error.code)
            return .failure(error)
        }
        guard let cString = result else {
            fatalError("ffi returned no error and null string");
        }

        defer { playgroundStringFree(cString)}

        let string = String(cString: cString, encoding: .utf8)!
        let message = try! JSONSerialization.jsonObject(with: string.data(using: .utf8)!)
        return .success(message)
    }

}

//
//  CompilerTask.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-04-23.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Foundation

/// Description of a task to be passed to rust
struct CompilerTask {

    enum TaskType: String {
        case run, check, test
    }

    let toolchain: String
    let code: String
    let type: TaskType

    var backtrace: Bool = true
    var release: Bool = false

    func toJson() -> String {
        let json: [String: Any] = [
            "toolchain": toolchain,
            "code": code,
            "task_type": type.rawValue,
            "backtrace": backtrace,
            "release": release
        ]
         let data = try! JSONSerialization.data(withJSONObject: json, options: [])
        return String(data: data, encoding: .utf8)!
    }
}

struct CompilerResult {
    let success: Bool
    let executable: String?
    let stdOut: String
    let stdErr: String

    static func fromJson(_ json: [String: AnyObject]) -> CompilerResult {
        let stdOut = json["stdout"] as! String
        let stdErr = json["stderr"] as! String
        let success = json["success"] as! Bool
        let executable = json["executable"] as? String
        return CompilerResult(success: success, executable: executable, stdOut: stdOut, stdErr: stdErr)
    }
}

var searchIndex = {};
searchIndex["rustypy"] = {"doc":"Binding Rust with Python, both ways!","items":[[3,"KrateData","rustypy","",null,null],[5,"parse_src","","",null,null],[5,"krate_data_new","","",null,null],[5,"krate_data_free","","",null,null],[5,"krate_data_len","","",null,{"inputs":[{"name":"kratedata"}],"output":{"name":"size_t"}}],[5,"krate_data_iter","","",null,null],[0,"pytypes","","Types for interfacing with Python.",null,null],[4,"PyArg","rustypy::pytypes","Enum type used to construct PyTuple types. All the kinds supported in Python\nare included here.",null,null],[13,"I64","","",0,null],[13,"I32","","",0,null],[13,"I16","","",0,null],[13,"I8","","",0,null],[13,"U32","","",0,null],[13,"U16","","",0,null],[13,"U8","","",0,null],[13,"F32","","",0,null],[13,"F64","","",0,null],[13,"PyBool","","",0,null],[13,"PyString","","",0,null],[13,"PyTuple","","",0,null],[0,"pystring","","An analog of a Python String.",null,null],[3,"PyString","rustypy::pytypes::pystring","An analog of a Python String.",null,null],[5,"PyString_free","","Destructs the PyString, mostly to be used from Python.",null,null],[5,"PyString_new","","Creates a PyString wrapper from a raw c_char pointer",null,null],[5,"PyString_get_str","","Consumes the wrapper and returns a raw c_char pointer. Afterwards is not necessary\nto destruct it as it has already been consumed.",null,null],[11,"clone","","",1,null],[11,"fmt","","",1,null],[11,"from_ptr","","Dereferences a PyString raw pointer to an inmutable reference.",1,null],[11,"to_string","","Constructs an owned String from a PyString.",1,null],[11,"from_ptr_to_string","","Constructs an owned String from a raw pointer.",1,null],[11,"as_ptr","","Returns PyString as a raw pointer. Use this whenever you want to return\na PyString to Python.",1,null],[11,"fmt","","",1,null],[11,"from","","Copies a string slice to a PyString.",1,{"inputs":[{"name":"str"}],"output":{"name":"pystring"}}],[11,"from","","Copies a String to a PyString.",1,{"inputs":[{"name":"string"}],"output":{"name":"pystring"}}],[0,"pybool","rustypy::pytypes","Analog to a Python boolean type.",null,null],[3,"PyBool","rustypy::pytypes::pybool","Analog to a Python boolean type.",null,null],[5,"PyBool_free","","",null,null],[5,"PyBool_new","","",null,null],[5,"PyBool_get_val","","",null,{"inputs":[{"name":"pybool"}],"output":{"name":"i8"}}],[11,"clone","","",2,null],[11,"fmt","","",2,null],[11,"from_ptr","","Dereferences a PyBool raw pointer to an inmutable reference.",2,null],[11,"from_ptr_into_bool","","Creates a bool from a raw pointer to a PyBool.",2,null],[11,"to_bool","","Conversion from PyBool to bool.",2,null],[11,"as_ptr","","Returns PyBool as a raw pointer. Use this whenever you want to return\na PyBool to Python.",2,null],[11,"from","","",2,{"inputs":[{"name":"bool"}],"output":{"name":"pybool"}}],[11,"from","","",2,{"inputs":[{"name":"bool"}],"output":{"name":"pybool"}}],[11,"eq","","",2,null],[11,"not","","",2,null],[11,"bitand","","",2,null],[11,"bitand","","",2,null],[11,"bitor","","",2,null],[11,"bitor","","",2,null],[0,"pytuple","rustypy::pytypes","An analog of a Python tuple, will accept an undefined number of other supported types.",null,null],[3,"PyTuple","rustypy::pytypes::pytuple","An analog of a Python tuple, will accept an undefined number of other supported types.",null,null],[12,"elem","","",3,null],[12,"idx","","",3,null],[12,"next","","",3,null],[5,"PyTuple_free","","",null,null],[5,"PyTuple_len","","",null,null],[5,"PyTuple_extractPyInt","","",null,null],[5,"PyTuple_extractPyBool","","",null,null],[5,"PyTuple_extractPyFloat","","",null,null],[5,"PyTuple_extractPyDouble","","",null,null],[5,"PyTuple_extractPyString","","",null,null],[11,"fmt","","",3,null],[11,"fmt","rustypy::pytypes","",0,null],[11,"fmt","rustypy","",4,null],[11,"visit_fn","","",4,null],[11,"visit_name","","",4,null],[14,"pytuple!","","This macro allows the construction of [PyTuple](../rustypy/pytypes/struct.PyTuple.html) types.",null,null]],"paths":[[4,"PyArg"],[3,"PyString"],[3,"PyBool"],[3,"PyTuple"],[3,"KrateData"]]};
initSearch(searchIndex);

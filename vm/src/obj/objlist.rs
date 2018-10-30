use super::super::pyobject::{
    AttributeProtocol, PyContext, PyFuncArgs, PyObject, PyObjectKind, PyObjectRef, PyResult,
    TypeProtocol,
};
use super::super::vm::VirtualMachine;
use super::objbool;
use super::objint;
use super::objiter;
use super::objsequence::{get_item, seq_equal, PySliceableSequence};
use super::objstr;
use super::objtype;
use num_bigint::ToBigInt;
use num_traits::ToPrimitive;

// set_item:
pub fn set_item(
    vm: &mut VirtualMachine,
    l: &mut Vec<PyObjectRef>,
    idx: PyObjectRef,
    obj: PyObjectRef,
) -> PyResult {
    if objtype::isinstance(&idx, &vm.ctx.int_type()) {
        let value = objint::get_value(&idx).to_i32().unwrap();
        let pos_index = l.get_pos(value);
        l[pos_index] = obj;
        Ok(vm.get_none())
    } else {
        panic!(
            "TypeError: indexing type {:?} with index {:?} is not supported (yet?)",
            l, idx
        )
    }
}

pub fn get_elements(obj: &PyObjectRef) -> Vec<PyObjectRef> {
    if let PyObjectKind::List { elements } = &obj.borrow().kind {
        elements.to_vec()
    } else {
        panic!("Cannot extract list elements from non-list");
    }
}

fn list_new(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(cls, None)],
        optional = [(iterable, None)]
    );

    if !objtype::issubclass(cls, &vm.ctx.list_type()) {
        return Err(vm.new_type_error(format!("{:?} is not a subtype of list", cls)));
    }

    let elements = match iterable {
        None => vec![],
        Some(iterable) => {
            let iterator = objiter::get_iter(vm, iterable)?;
            objiter::get_all(vm, &iterator)?
        }
    };

    Ok(PyObject::new(
        PyObjectKind::List { elements: elements },
        cls.clone(),
    ))
}

fn list_eq(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(zelf, Some(vm.ctx.list_type())), (other, None)]
    );

    let result = if objtype::isinstance(other, &vm.ctx.list_type()) {
        let zelf = get_elements(zelf);
        let other = get_elements(other);
        seq_equal(vm, zelf, other)?
    } else {
        false
    };
    Ok(vm.ctx.new_bool(result))
}

fn list_add(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(o, Some(vm.ctx.list_type())), (o2, None)]
    );

    if objtype::isinstance(o2, &vm.ctx.list_type()) {
        let e1 = get_elements(o);
        let e2 = get_elements(o2);
        let elements = e1.iter().chain(e2.iter()).map(|e| e.clone()).collect();
        Ok(vm.ctx.new_list(elements))
    } else {
        Err(vm.new_type_error(format!("Cannot add {:?} and {:?}", o, o2)))
    }
}

fn list_repr(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(o, Some(vm.ctx.list_type()))]);

    let elements = get_elements(o);
    let mut str_parts = vec![];
    for elem in elements {
        let s = vm.to_repr(elem)?;
        str_parts.push(objstr::get_value(&s));
    }

    let s = format!("[{}]", str_parts.join(", "));
    Ok(vm.new_str(s))
}

pub fn list_append(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    trace!("list.append called with: {:?}", args);
    arg_check!(
        vm,
        args,
        required = [(list, Some(vm.ctx.list_type())), (x, None)]
    );
    let mut list_obj = list.borrow_mut();
    if let PyObjectKind::List { ref mut elements } = list_obj.kind {
        elements.push(x.clone());
        Ok(vm.get_none())
    } else {
        Err(vm.new_type_error("list.append is called with no list".to_string()))
    }
}

fn list_clear(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    trace!("list.clear called with: {:?}", args);
    arg_check!(vm, args, required = [(list, Some(vm.ctx.list_type()))]);
    let mut list_obj = list.borrow_mut();
    if let PyObjectKind::List { ref mut elements } = list_obj.kind {
        elements.clear();
        Ok(vm.get_none())
    } else {
        Err(vm.new_type_error("list.clear is called with no list".to_string()))
    }
}

pub fn list_extend(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(list, Some(vm.ctx.list_type())), (x, None)]
    );
    let mut new_elements = vm.extract_elements(x)?;
    let mut list_obj = list.borrow_mut();
    if let PyObjectKind::List { ref mut elements } = list_obj.kind {
        elements.append(&mut new_elements);
        Ok(vm.get_none())
    } else {
        Err(vm.new_type_error("list.extend is called with no list".to_string()))
    }
}

fn list_len(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    trace!("list.len called with: {:?}", args);
    arg_check!(vm, args, required = [(list, Some(vm.ctx.list_type()))]);
    let elements = get_elements(list);
    Ok(vm.context().new_int(elements.len().to_bigint().unwrap()))
}

fn list_reverse(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    trace!("list.reverse called with: {:?}", args);
    arg_check!(vm, args, required = [(list, Some(vm.ctx.list_type()))]);
    let mut list_obj = list.borrow_mut();
    if let PyObjectKind::List { ref mut elements } = list_obj.kind {
        elements.reverse();
        Ok(vm.get_none())
    } else {
        Err(vm.new_type_error("list.reverse is called with no list".to_string()))
    }
}

fn list_contains(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    trace!("list.contains called with: {:?}", args);
    arg_check!(
        vm,
        args,
        required = [(list, Some(vm.ctx.list_type())), (needle, None)]
    );
    for element in get_elements(list).iter() {
        match vm.call_method(needle, "__eq__", vec![element.clone()]) {
            Ok(value) => {
                if objbool::get_value(&value) {
                    return Ok(vm.new_bool(true));
                }
            }
            Err(_) => return Err(vm.new_type_error("".to_string())),
        }
    }

    Ok(vm.new_bool(false))
}

fn list_getitem(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    trace!("list.getitem called with: {:?}", args);
    arg_check!(
        vm,
        args,
        required = [(list, Some(vm.ctx.list_type())), (needle, None)]
    );
    get_item(vm, list, &get_elements(list), needle.clone())
}

pub fn init(context: &PyContext) {
    let ref list_type = context.list_type;
    list_type.set_attr("__add__", context.new_rustfunc(list_add));
    list_type.set_attr("__contains__", context.new_rustfunc(list_contains));
    list_type.set_attr("__eq__", context.new_rustfunc(list_eq));
    list_type.set_attr("__getitem__", context.new_rustfunc(list_getitem));
    list_type.set_attr("__len__", context.new_rustfunc(list_len));
    list_type.set_attr("__new__", context.new_rustfunc(list_new));
    list_type.set_attr("__repr__", context.new_rustfunc(list_repr));
    list_type.set_attr("append", context.new_rustfunc(list_append));
    list_type.set_attr("clear", context.new_rustfunc(list_clear));
    list_type.set_attr("extend", context.new_rustfunc(list_extend));
    list_type.set_attr("reverse", context.new_rustfunc(list_reverse));
}

#[macro_export]
macro_rules! tyenum {
    ($name:ident {
        $( $key:ident { $val_name:ident: $val:ty } ),*
    }) => {
        #[allow(dead_code, non_camel_case_types)]
        #[repr(u8)]
        enum $name<K: 'static> {
            $( $key($val) ),*,
            _hidden(::std::marker::PhantomData<K>)
        }
        
        #[allow(non_camel_case_types)]
        impl<K: 'static> $name<K> {
            pub fn new<V: 'static>(v: V) -> Self {
                use std::any::TypeId;
                use std::marker::PhantomData;
                use std::ptr;
                
                assert!(
                    $( (TypeId::of::<K>() == TypeId::of::<$key>()
                        && TypeId::of::<V>() == TypeId::of::<$val>()) )||*
                );
                
                let mut out = $name::_hidden(PhantomData);
                unsafe { ptr::write(out.inner_mut(), v) }
                out
            }
            
            unsafe fn inner<V: 'static>(&self) -> *const V {
                #[repr(C)]
                struct Repr<V> {
                    _discriminiant: u8,
                    _inner: V
                }
                &(*(self as *const Self as *const Repr<V>))._inner
            }
            
            unsafe fn inner_mut<V: 'static>(&mut self) -> *mut V {
                #[repr(C)]
                struct Repr<V> {
                    _discriminiant: u8,
                    _inner: V
                }
                &mut (*(self as *mut Self as *mut Repr<V>))._inner
            }
        
            
            #[allow(dead_code)]
            pub fn match_move
                <$($val_name),*, Out>
                (mut self, $($val_name: impl FnOnce($val_name) -> Out),*)
                -> Out
            where
                $( $val_name: 'static ),*
            {
                use std::any::TypeId;
                use std::{mem, ptr};
                $(
                    if TypeId::of::<K>() == TypeId::of::<$key>()
                        && TypeId::of::<$val_name>() == TypeId::of::<$val>() {
                        
                        let cast_self = unsafe { ptr::read(self.inner_mut()) };
                        mem::forget(self);
                        return $val_name(cast_self);
                    }
                ) else *
                unreachable!();
            }
            
            
            #[allow(dead_code)]
            pub fn match_ref
                <$($val_name),*, Out>
                (&self, $($val_name: impl FnOnce(&$val_name) -> Out),*)
                -> Out
            where
                $( $val_name: 'static ),*
            {
                use std::any::TypeId;
                $(
                    if TypeId::of::<K>() == TypeId::of::<$key>()
                        && TypeId::of::<$val_name>() == TypeId::of::<$val>() {
                        
                        let cast_self = unsafe { &*self.inner() };
                        return $val_name(cast_self);
                    }
                ) else *
                unreachable!();
            }
            
            
            #[allow(dead_code)]
            pub fn match_ref_mut
                <$($val_name),*, Out>
                (&mut self, $($val_name: impl FnOnce(&mut $val_name) -> Out),*)
                -> Out
            where
                $( $val_name: 'static ),*
            {
                use std::any::TypeId;
                $(
                    if TypeId::of::<K>() == TypeId::of::<$key>()
                        && TypeId::of::<$val_name>() == TypeId::of::<$val>() {
                        
                        let cast_self = unsafe { &mut *self.inner_mut() };
                        return $val_name(cast_self);
                    }
                ) else *
                unreachable!();
            }
        }
        
        impl<K: 'static> Drop for $name<K> {
            fn drop(&mut self) {
                use std::ptr::drop_in_place;
                
                self.match_ref_mut(
                    $( |k: &mut $val| unsafe { drop_in_place(k) } ),*
                )
            }
        }
    }
}



#[test]
fn test_match() {
    tyenum!(Number {
        Marker1 { _i32: i32 },
        Marker2 { _i64: i64 }
    });
    
    trait Marker: 'static {}
    struct Marker1; impl Marker for Marker1 {}
    struct Marker2; impl Marker for Marker2 {}
    
    /// Simplified: returns -2(N + {12i32|13i64})
    fn do_op<M: Marker>(mut num: Number<M>) -> i32 {
        num.match_ref_mut(
            |k: &mut i32| *k += 12,
            |k: &mut i64| *k += 13,
        );
    
        let neg = num.match_ref(
            |k: &i32| -k,
            |k: &i64| -k as i32,
        );
    
        neg - num.match_move(
            |k: i32| k,
            |k: i64| k as i32,
        )
    }
    
    let num = Number::<Marker1>::new(5i32);
    assert!(do_op(num) == -34);
    let num2 = Number::<Marker2>::new(5i64);
    assert!(do_op(num2) == -36);
}

#[test]
fn test_drop() {
    use std::rc::Rc;
    use std::cell::Cell;
    
    struct Droppable {
        drop_flag: Rc<Cell<bool>>
    }
    impl Drop for Droppable {
        fn drop(&mut self) {
            self.drop_flag.set(true);
        }
    }
    
    struct NotDroppable;

    tyenum!(TyEnum {
        Marker1 { _d: Droppable },
        Marker2 { _nd: NotDroppable }
    });
    
    trait Marker: 'static {}
    struct Marker1; impl Marker for Marker1 {}
    struct Marker2; impl Marker for Marker2 {}
    
    let drop_flag = Rc::new(Cell::new(false));
    {
        let _val = TyEnum::<Marker1>::new(Droppable { drop_flag: drop_flag.clone() });
        assert!(!drop_flag.get());
    }
    assert!(drop_flag.get());
}
